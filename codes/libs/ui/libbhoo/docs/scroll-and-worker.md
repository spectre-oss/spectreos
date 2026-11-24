# Scroll physics, navigation motion and worker rendering

## EdgeStretch overscroll physics

Implemented from AOSP `EdgeEffect.java` `TYPE_STRETCH` constants. `codes/libs/ui/libbhoo/src/scroll.rs` defines pull, reverse‑pull, damping, recoil, and fling‑absorb mechanics. The `Frame::with_edge_stretch` and `Frame::scroll_stretch` APIs expose this behavior.

## Frame::cancel_pointer_input

Handles `ACTION_CANCEL` allowing ripples to finish their 375 ms fade while cancelling pending tap ripples (<150 ms). This is used in drag interactions and overscroll cancellation.

## OverviewGlide velocity‑matched deceleration

Used in the Launcher Overview for smooth card deceleration matching user swipe velocity.

## ScrollPhysics integration

`bhoo::scroll::ScrollPhysics` incorporates the overscroll stretch and fling mechanics, providing realistic inertial scrolling.

---



A fling follows velocity and friction, rather than replaying a fixed Material
easing curve. `bhoo::scroll::ScrollPhysics` uses the AOSP OverScroller spline
with inflexion 0.35, tension controls 0.175/0.35, deceleration rate
`ln(0.78)/ln(0.9)`, friction 0.015, and the gravity-derived physical coefficient
`9.80665 × 39.37 × 160 × 0.84` for logical-pixel density 1.
Sampling uses elapsed time, not frame count. Velocity uses finger samples and
clears after a 100ms hold. Touch interrupts a fling; cancellation/blur clears it.

The page and `scroll_viewport` containers use resisted edge displacement and an
analytic critically damped return spring (mass 1, stiffness 324, damping 36).
This supplies elastic edge feedback; it is **not** an exact Android 12+
`EdgeEffect` stretch shader. Existing clipped overflow menus stop at their bounds.
A `SplineFling` adapter feeds existing toolbar, button group and top/bottom app
bar nested-scroll chains. Lists/grids, scroll viewport and FAB menu, the sidebar, source dialog and imported
XML ScrollViews use the shared viewport controller. Wheel/trackpad input is
normalized and coalesced by the host; it eases toward its accumulated target.
Carousel snapping retains its component-specific spatial motion.

Programmatic gallery navigation interpolates to its destination over 500ms.
The desktop rail width and mobile drawer geometry use Material Standard easing
over 300ms; sidebar selection uses 200ms, bottom navigation indicators 300ms.
The default library components `material_navigation_item` and
`material_navigation_bar_item` provide Material icons, state layers, ripples,
focus indication, selected semantics and animated selection. Reduced motion
snaps transitions and suppresses inertial continuation/edge movement.

References:
- [Android OverScroller API](https://developer.android.com/reference/android/widget/OverScroller)
- [AOSP OverScroller source](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/widget/OverScroller.java)
- [AOSP EdgeEffect source](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/widget/EdgeEffect.java)
- [Material easing and duration](https://m3.material.io/styles/motion/easing-and-duration/applying-easing-and-duration)

## Web rendering

`single-canvas.js` transfers the visible canvas to a module worker when supported.
`canvas-worker.js` hosts `canvas-runtime.js`: one WASM instance, retained Bhoo
contexts, Rust layout/hit testing/motion, and Canvas 2D rendering. The main thread
only handles trusted DOM events, pointer capture, IME and accessibility/debug
metadata. No pixel buffers or full render packets cross the worker boundary.
A capability/startup fallback runs the same runtime on the main thread. Add
`?main` to exercise that path explicitly.

Only visible surfaces are drawn. Settled surfaces use a bounded 24 MiB raster
cache, image data uses a separate 16 MiB cap, device pixel ratio is capped at 2,
and redundant motion events are coalesced. Animating, focused/input-targeted or
invalidated surfaces repaint. Static/hidden documents stop scheduling frames.
Unsupported OffscreenCanvas environments use detached canvas caches.

The worker improves main-thread headroom; it does not remove GPU fill cost or
promise 60/120 FPS on every phone. Browser emulation is not physical-device proof.
The small separate XML demo uses the shared packet painter on the main thread.

## Checks

- Rust spline travel agrees at 60 and 120Hz; edges settle and reduced motion stops.
- Existing navigation, toolbar/menu and nested app-bar tests verify interruption,
  keyboard access, boundary handling and click cancellation.
- `tools/test-bhoo-single-canvas.mjs`: 97 examples, one visible canvas/one WASM,
  worker-aware resource checks, desktop/phone touch, keyboard, IME, retained state.
- `tools/profile-bhoo-canvas-scroll.mjs`: CPU render timing and cache budget;
  use `BHOO_PROFILE_ENTRY` and `BHOO_PROFILE_MOBILE` for worker/fallback phone runs.

Phone-sized Chromium run (390×844, DPR 3 capped to 2, 2026-11-11): worker
median render CPU 1.6ms without cache / 0.5ms with cache; p95 2.1ms / 1.7ms;
cache 5,818,368 bytes. Main-thread sampled frame interval median 16.7ms in both.
These are development-host measurements, not physical-phone FPS measurements.
