# Android XML MediaController

Status: Initial XML container with host-provided player state and actions. Initial anchor-bound floating mode is available; native windows, real player integration and Android device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

An XML MediaController is a FrameLayout whose supplied descendant controls use Android's standard IDs: `pause`, `rew`, `ffwd`, `prev`, `next`, `time_current`, `time`, and `mediacontroller_progress`. A host snapshot supplies current position, duration, buffering, and playing state. The example provides a custom XML layout, Android 15 framework media icons and a simulated player. The play icon switches with player state. The host can show the same controls at the bottom of an anchor in a canvas overlay. Native windows and full default style are not yet implemented.

## State and interaction contract

Controls report play, pause, previous, next and seek requests to the host. Rewind requests five seconds back; fast-forward requests fifteen seconds ahead. The host owns playback and applies resulting state. Progress uses the framework's 0–1000 range. During a pointer seek the thumb and elapsed-time preview stay at the dragged position even if the host snapshot has not yet updated; after release the host snapshot is authoritative. This implementation does not play media or access a media session. Separate controllers scope standard child control identities and restore host text/progress values after rendering. Pause, rewind and fast-forward follow player capabilities; the seek bar is disabled only if neither seek direction is available. Previous and next follow installed host listeners. More complex nested layout scoping remains open.

## Motion and animation

Inline XML controllers stay visible. Floating controls hide after the requested timeout (three seconds by default in the example), remain visible while seeking, and restart the timeout after interaction. A zero timeout keeps them open until hidden. Android window and theme details remain open.

**Reduced motion:** No controller-specific motion is implemented.

## Acceptance criteria

- Bind the standard XML child IDs and report playback actions.
- Show elapsed and total time and seek progress from the host snapshot.
- Implement native floating windows, remaining capability details, actual player integration and compare with Android.

## References

- [Android 15 MediaController source](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/MediaController.java) and [default layout](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/media_controller.xml), Apache-2.0.
