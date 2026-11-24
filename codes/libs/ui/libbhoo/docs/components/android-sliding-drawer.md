# Android XML SlidingDrawer

Status: Initial implementation; exact Android physics and device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (deprecated) |
| Priority | P1 |

## Scope and variants

The XML importer accepts a SlidingDrawer with distinct, uniquely identified handle and content descendants, vertical or horizontal orientation, offsets, allowSingleTap and animateOnClick. Bhoo locates and renders the referenced views directly; ancestor-wrapper layout and state are not reproduced for nested references.

## State and interaction contract

Tap the handle to open or close, or drag it along its axis; a slow drag settles based on position, while a fast fling can change the result. The handle can also be activated by keyboard. Child content can receive input while visible. The host can lock user gestures and request immediate or animated opening and closing; programmatic requests still work while locked. The drawer queues `SlidingDrawerEvent::ScrollStarted` on handle press and `ScrollEnded` on release or cancellation; animated programmatic requests enqueue start and end together when processed, matching the framework listener timing. These mark tracking or request initiation, not the end of settling. Completed state changes enqueue `Opened` or `Closed`; hosts drain all events with `LayoutState::take_sliding_drawer_events`. Immediate changes report on the next render, animated changes when the destination is reached, and repeated requests for the current state do not duplicate events. The queue retains the latest 32 events. Multi-pointer and exact platform velocity tracking, ancestor-wrapper effects and Android device parity remain open. Bhoo cancels active tracking when locked and reports `ScrollEnded`; Android lock-during-tracking behavior still requires comparison.

## Motion and animation

The handle and clipped content move together with acceleration and an estimate of release velocity. Fling direction and thresholds follow the Android source; the input velocity estimator and timing still need device comparison.

**Reduced motion:** The drawer settles immediately.

## Acceptance criteria

- Import the handle and content and operate the drawer in both orientations.
- Implement Android fling and locking behavior and compare against Android.

## References

- [Android 15 SlidingDrawer source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/SlidingDrawer.java)

## Validation and source

Importer tests cover nested IDs, missing or duplicate targets, and identical IDs. Rust tests exercise nested handles and an interactive content button in both orientations as well as the existing drag, tap, fling and lock paths, including state and tracking event order, cancellation, locking, interruption, deduplication and bounded retention. A 320 × 760 browser test opens the nested example and activates its content button in both orientations. Separate phone-sized browser tests check open/close event text and tracking event order for touch and locked programmatic changes. Device rendering and nested-ancestor parity remain unverified. The [Android 15 SlidingDrawer source](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/SlidingDrawer.java) is Apache-2.0 and available locally at `context/reference-sources/android-framework/SlidingDrawer.java`.
