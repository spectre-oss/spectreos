# Android XML ViewSwitcher

Status: Initial XML child switching support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android ViewSwitcher shows one of up to two child views inside a frame.

## State and interaction contract

The host owns the displayed child index by view ID. The first child is shown initially; switching preserves child state. ViewAnimator also supports host-selected children. Factories and full Android measurement are not yet supported.

## Motion and animation

XML alpha, translate, Wave 16A/16B `<scale>` and `<rotate>` (`RULE_SCALE_IN = 274`, `RULE_SCALE_OUT = 275`, `RULE_ROTATE_IN = 276`, `RULE_ROTATE_OUT = 277`, emitted from `res/anim/*.xml` `<scale>`/`<rotate>` and `res/animator/*.xml` `scaleX`/`scaleY`/`rotation`), and Wave 19A/19B `elevation` / `translationZ` object animators (`RULE_ELEVATION_IN = 288`, `RULE_ELEVATION_OUT = 289`, `"<from_dp>,<to_dp>|<dur_ms>|<offset_ms>|<interp>"`, supporting sequential/together `<set>` combining `elevation` + `translationZ`, density-aware `px` -> `dp` scaling, and all 7 `AnimInterp` curves mapped by `parse_interpolator_kind`: `linear`, `accelerate`, `decelerate`, `fast_out_slow_in`, `overshoot`, `anticipate`, `accelerate_decelerate`) can run independently or together in sets on `ViewAnimator` and `ViewSwitcher`. Scale and rotate transitions wrap incoming/outgoing children in balanced `DrawCommand::PushTransform` / `PopTransform` calls honoring fixed (`dp`), child-relative (`%`), parent-relative (`%p`), and `center` pivots; elevation transitions continuously interpolate `paint_elevation_dp` on incoming/outgoing children while suppressing static child `RULE_ELEVATION` during active transitions to prevent double shadows. Translation supports fixed, child-relative and parent-relative offsets. The importer supports `android:startOffset` on alpha, translate, scale, rotate, elevation, and nested sets, adding set and child delays; before the delay expires the initial transform remains applied. The transition remains active until the longest delay plus duration. The limit is 60 seconds for the accumulated delay. Duplicate animation types in a set, arbitrary custom `<pathInterpolator>` curves, `repeatCount`/`repeatMode` runtime playback, fill options (`fillBefore`/`fillAfter`) and interruption parity remain unsupported. ObjectAnimator start delays for adapter rows are handled by AdapterViewFlipper.

**Reduced motion:** Child changes become immediate (no alpha, translate, scale, or rotate transition, and `RULE_ELEVATION_IN`/`RULE_ELEVATION_OUT` snap immediately to `to_dp`).

## Acceptance criteria

- Reject more than two ViewSwitcher children.
- Show only the host-selected child and preserve state.
- Compare alpha, translate, scale, rotate, and elevation transitions with Android and add remaining animation resources.

## References

- [Android ViewSwitcher source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/master/core/java/android/widget/ViewSwitcher.java)
- [Android ViewAnimator reference](https://developer.android.com/reference/android/widget/ViewAnimator)

## Validation and source

Importer tests ([`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), `241` passing tests, including Wave 16B `test_wave16b_scale_and_rotate_transition_rules_274_to_277` and Wave 19B `test_wave19b_elevation_and_translation_z_animators_rules_288_289`) cover nested set offsets, independent delays, `<scale>`/`<rotate>` rule `274..=277` and `elevation`/`translationZ` rule `288..=289` emission across `ViewAnimator`/`ViewSwitcher`/`TextSwitcher`/`ImageSwitcher`/`ViewFlipper`/`AdapterViewFlipper`, all 7 `AnimInterp` curves, density-aware `px` -> `dp` scaling, and invalid offsets. Rust rendering tests ([`tests/android_xml.rs`](../../tests/android_xml.rs), `312` passing tests; `907` default / `913` all-features Rust tests across `76` suites, including Wave 16A `wave16a_scale_and_rotate_transition_animations_and_reduced_motion` and Wave 19A `wave19a_elevation_object_animator_transitions_and_reduced_motion`) check alpha, translation, scale, rotation (`PushTransform`/`PopTransform`), and continuous elevation (`paint_elevation_dp`, suppressing duplicate static child shadows) at intermediate times, transition lifetime, and reduced-motion immediate cuts/snaps. The focused gallery example switches between two colored views after an 800 ms delay, followed by a 500 ms slide and fade. A 320 × 440 browser pixel test checks that the view remains unchanged during the delay and changes after the animation. Android-device visual and timing parity remain unverified.

The delay behavior is based on Android 15 Apache-2.0 sources: [Animation](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/view/animation/Animation.java) and [AnimationSet](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/view/animation/AnimationSet.java), saved in `context/reference-sources/android-framework/Animation.java` and `AnimationSet.java`.
