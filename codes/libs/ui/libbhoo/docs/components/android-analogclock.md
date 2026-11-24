# Android XML AnalogClock

Status: Initial host-time analog clock support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Deprecated Android framework widget |
| Priority | P2 |

## Scope and variants

AnalogClock displays local wall time with hour and minute hands.

## State and interaction contract

The host supplies Unix time and local UTC offset. The dial and hands use sampled meshes from pinned AOSP framework vector resources. Dynamic Android color palettes, custom dial and hand drawables, optional second hand and custom time zone remain unsupported.

## Motion and animation

The clock schedules updates on minute boundaries without continuous redraw.

**Reduced motion:** Time updates remain available.

## Acceptance criteria

- Render the hands from the host wall clock and offset.
- Schedule the next minute update.
- Compare Android drawables and timing on a device.

## References

- [Android AnalogClock reference](https://developer.android.com/reference/android/widget/AnalogClock)
- [Android AnalogClock source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/master/core/java/android/widget/AnalogClock.java)
