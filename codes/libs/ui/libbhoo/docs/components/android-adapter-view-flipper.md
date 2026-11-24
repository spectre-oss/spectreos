# Android AdapterViewFlipper

Status: Initial adapter-backed XML flipper with default crossfade. Custom alpha, translation, Wave 16A/16B `scaleX`/`scaleY`/`rotation` (`RULE_SCALE_IN = 274..=RULE_ROTATE_OUT = 277`), and Wave 19A/19B `elevation`/`translationZ` object animators (`RULE_ELEVATION_IN = 288`, `RULE_ELEVATION_OUT = 289`, 7 `AnimInterp` curves) are supported; `repeatCount`/`repeatMode` runtime playback, arbitrary custom `<pathInterpolator>` curves, RemoteViews adapters, and Android device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The XML importer accepts `AdapterViewFlipper`, `autoStart`, `flipInterval`, and `loopViews`, with Android's 10-second default interval. The host provides adapter labels and an optional noninteractive XML row template. The flipper wraps at both ends: Android's AdapterViewFlipper constructor forces looping even if XML specifies `loopViews="false"`.

## State and interaction contract

The host can start or stop automatic flipping, advance or reverse manually, and receive the clicked adapter position. A manual change restarts the timer. Empty adapters do not schedule flips. Host-driven mode stops the internal timer while retaining the started state and allows explicit advancement. When data changes, a valid displayed position is retained, an out-of-range position resets to zero, and a press cannot activate a replacement item.

## Motion and animation

Changing items uses the framework's default 200 ms crossfade. XML alpha, horizontal and vertical translation, Wave 16A/16B `scaleX`/`scaleY`/`rotation` (`RULE_SCALE_IN = 274`, `RULE_SCALE_OUT = 275`, `RULE_ROTATE_IN = 276`, `RULE_ROTATE_OUT = 277`), and Wave 19A/19B `elevation` / `translationZ` (`RULE_ELEVATION_IN = 288`, `RULE_ELEVATION_OUT = 289`, `"<from_dp>,<to_dp>|<dur_ms>|<offset_ms>|<interp>"`) object animators can run together via balanced `DrawCommand::PushTransform` / `PopTransform` calls and continuous `paint_elevation_dp` interpolation (suppressing static row-template `RULE_ELEVATION` during active transitions to prevent double shadows). `parse_interpolator_kind` maps all 7 `AnimInterp` curves (`linear`, `accelerate`, `decelerate`, `fast_out_slow_in`, `overshoot`, `anticipate`, `accelerate_decelerate`), and `object_alpha_animator` supports sequential/together `<set>` combining `elevation` + `translationZ` as well as density-aware `px` -> `dp` scaling. Each property can use its own duration, interpolator and `android:startOffset` (up to 60 seconds). During the delay Bhoo keeps alpha at 1 and translation at zero; each axis starts independently. This approximates Android object-animator behavior for newly presented views; retained properties and interruption still need device validation. Bhoo also accepts together and sequential `<set>` combinations in `res/animator/`, although the Android 15 `AdapterViewAnimator` XML path casts the loaded animator to `ObjectAnimator`, so set compatibility with that framework path is not claimed. Runtime `repeatCount`/`repeatMode` playback, arbitrary custom `<pathInterpolator>` curves, remote adapters and lifecycle details remain open. Android 15 stores `animateFirstView` but its adapter animator does not consult it on the initial adapter display.

**Reduced motion:** Item changes are immediate (skipping alpha, translation, scale, and rotation transitions, and snapping `RULE_ELEVATION_IN`/`RULE_ELEVATION_OUT` immediately to `to_dp`).

## Acceptance criteria

- Advance, reverse, wrap, pause and resume with adapter data.
- Compare animation, lifecycle and remote adapter behavior on Android.

## References

- [Android 15 AdapterViewFlipper](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/AdapterViewFlipper.java).
- [Android 15 AdapterViewAnimator](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/AdapterViewAnimator.java).

## Validation and source

Importer tests ([`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), `241` passing tests, including Wave 16B `test_wave16b_scale_and_rotate_transition_rules_274_to_277` and Wave 19B `test_wave19b_elevation_and_translation_z_animators_rules_288_289`) cover independent axis offsets, `274..=277` and `288..=289` rule emission, all 7 `AnimInterp` curves, sequential/together `elevation` + `translationZ` sets, and density-aware `px` -> `dp` scaling; Rust tests ([`tests/android_xml.rs`](../../tests/android_xml.rs), `312` passing tests; `907` default / `913` all-features Rust tests across `76` suites, including Wave 16A `wave16a_scale_and_rotate_transition_animations_and_reduced_motion` and Wave 19A `wave19a_elevation_object_animator_transitions_and_reduced_motion`) cover delayed identity, axis timing, scale/rotate `PushTransform`/`PopTransform` transitions, continuous `paint_elevation_dp` interpolation with static shadow suppression, reduced motion, and flipper interactions. Android device motion and pixels are unverified. The focused example gives vertical translation a 500 ms delay. References: [Android 15 AnimatorInflater](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/animation/AnimatorInflater.java) and [ValueAnimator](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/animation/ValueAnimator.java) (Apache-2.0); local copies are `context/reference-sources/android-framework/AnimatorInflater.java` and `ValueAnimator.java`.
