# Android XML VideoView

Status: Host-frame surface and linked floating controller. The standard browser test page decodes a bundled test video offscreen; native decoder integration and device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The XML importer accepts VideoView. The host supplies decoded frames and reports whether the player is prepared. Frames fit within the view without cropping; an empty surface is black. Android’s exact, at-most and unspecified aspect-ratio measurement rules are implemented; FrameLayout and LinearLayout child placement use them, including remeasuring weighted video children and positioning subsequent siblings. Direct host-provided bounds, advanced LinearLayout options and other parent layout measurement paths still require Android parity, as does SurfaceView compositing.

## State and interaction contract

When a prepared VideoView is linked to a MediaController, a press on the surface toggles the floating controller. Presses inside an open controller are reserved for its controls. The host supplies playback state and handles controller actions. The standard browser test page uses a detached HTML video decoder to supply frames and player state to Bhoo; the visible controls and video surface remain on the canvas. The bundled silent test pattern is loaded locally and made seekable as a Blob, with decoded frames bounded to 320 × 180 pixels. The bridge pauses when the page is hidden. Other hosts must supply their own decoder and player.

## Motion and animation

The linked controller uses its timed floating behavior; frame delivery is owned by the host.

**Reduced motion:** Frame delivery and controller visibility do not add motion.

## Acceptance criteria

- Import a VideoView, paint supplied frames and toggle a linked controller.
- Implement native-host decoder and player integration, remaining parent measurement paths and compare with Android.

## References

- [Android 15 VideoView source](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/VideoView.java) (Apache-2.0).
