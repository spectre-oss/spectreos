# Android XML ZoomControls

Status: Initial support for the deprecated Android framework widget.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P2 |

## Scope and variants

ZoomControls contains a zoom-out and a zoom-in button. Android deprecated this widget in favor of custom controls.

## State and interaction contract

The XML importer accepts `ZoomControls`. Each button supports tap and long-press repeat. The host receives the parent view ID and reads `LayoutState.zoom_controls_actions` for the direction (-1 or +1). `zoom_controls_enabled` controls each button independently and `zoom_speed_ms` sets the repeat interval for both. Buttons follow layout direction. The default buttons use Android 15 framework mdpi drawables for normal, pressed, focused and disabled states. The host can animate visibility through `LayoutState::set_zoom_controls_visible`. The default mdpi assets preserve their nine-patch stretch regions when the buttons are constrained. Other Android themes and density-qualified resources remain unsupported.

## Motion and animation

Buttons repeat actions while held. Show and hide use a 500 ms accelerate-decelerate fade and can reverse from the displayed opacity; while hiding, input is disabled. This initial implementation removes the control from layout after the fade; Android platform layout timing still requires comparison.

**Reduced motion:** Visibility snaps to the target; repeat timing is unchanged.

## Acceptance criteria

- Import the XML tag and emit separate zoom-in and zoom-out actions.
- Disable either button independently and stop repeating on release or cancellation.
- Compare Android theme selection, constrained nine-patch sizing and show/hide layout timing on a device.

## References

- [Android ZoomControls reference](https://developer.android.com/reference/android/widget/ZoomControls)
- [Android 15 framework layout, Apache-2.0](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/zoom_controls.xml)
- Android 15 `ZoomControls.java` and `res/layout/zoom_controls.xml` in `context/reference-sources/android-framework/`, Apache-2.0.
