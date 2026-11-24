# Android XML seek bar

Status: Initial XML support using Bhoo's slider with integer min, max and progress, plus Wave 21A/21B timed `<animation-list>` `android:tickMark` playback (`RULE_COMPOUND_ANIMATION_LIST = 295`).

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

An Android XML SeekBar with an integer range, starting progress and enabled or disabled state.

## State and interaction contract

The caller owns the progress state by view ID. Dragging updates the value within the range; disabled controls remain unchanged. A separately controlled secondary progress track is drawn behind the primary track. Explicit `android:mirrorForRtl="true"` mirrors horizontal progress in RTL; theme-specific defaults are not yet resolved.

`android:progressTint`, `progressBackgroundTint`, `secondaryProgressTint`, `thumbTint` and `tickMarkTint` accept literal colors, color resources and supported color-state selectors. The selectors follow enabled, pressed, focused, selected, activated and hovered state. Missing tints retain Bhoo's default colors, including its disabled treatment. Explicit tints color Bhoo's existing track and circular thumb, so arbitrary Android drawable composition and blending are not reproduced.

`android:tickMark` accepts `@null`, a local solid XML shape with explicit dimensions, a selector of supported shapes and `@null`, or (Wave 21A/21B) a timed `<animation-list>` drawable (including `<inset>`-wrapped lists with cyclic-reference detection). Rectangles may have uniform rounded corners, and ovals must be circular. Selectors support enabled, pressed, focused, selected, activated and hovered states. For `<animation-list>`, `tools/import-android-layout.py` (`resolve_animation_list_drawable` + `decode_animation_list_frames`) bundles all frames into `install_drawables`, preserves frame-0 static fallback `kind: 155`, and emits `RULE_COMPOUND_ANIMATION_LIST = 295` (`"tick:<oneshot:0|1>|<img_key_0>:<dur_ms_0>,..."`); at runtime (`select_animation_list_frame`), `Kind::SeekBar` (and M3 `Slider` / `RangeSlider`) advances frames via `f.time()` and clip-gated `f.wake_at()`, looping (`oneshot = 0`) or clamping on the final frame with no wakeup (`oneshot = 1`), and freezing strictly on frame `0` with zero wakeup under `MotionPreference::Reduced`. For an integer range greater than one, a mark is painted at every value including both endpoints, below the thumb. The importer rejects tick ranges above 1024 and unsupported drawables instead of silently dropping them. Marks align with Bhoo's thumb travel; exact Android padding, intrinsic sizing and drawable parity have not been verified.

`android:thumb` accepts `@null`, solid XML shapes with explicit dimensions or state selectors of supported shapes and `@null`; rectangular shapes may have uniform corners and ovals must be circular. Supported selector states are enabled, pressed, focused, selected, activated and hovered. `thumbTint` and `tickMarkTint` apply the default source-in tint to selected solid shapes, multiplying the drawable and tint alpha. Explicit `thumbTintMode` and `tickMarkTintMode` support src_in, src_atop, src_over, multiply, screen and add for supported custom shapes; the two modes are independent. Other blend modes and antialiased edge compositing are not reproduced. Indeterminate mode and its tint, built-in thumb tint modes, bitmap/vector/animated `android:thumb` drawables, static bitmap/vector or `<animated-vector>`/`<animated-selector>` tick drawables, and theme attribute resolution remain unsupported.

Explicit `android:splitTrack="true"` clips the track and tick marks out of the current thumb bounds before drawing the thumb. The built-in thumb is 20 dp square; a supported custom shape supplies its own dimensions, and `@null` leaves the track continuous. The gap follows the thumb in RTL. The default is false. Android custom thumb optical insets, thumb offset and exact position calculation remain unsupported.

## Motion and animation

The thumb follows direct pointer manipulation using Bhoo's slider rendering. Timed `<animation-list>` `android:tickMark` drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`) advance by frame duration using `f.time()` and clip-gated `f.wake_at()` wakeups (stopping when offscreen or once a `oneshot` sequence finishes; see [motion.md](../motion.md#scheduling-and-lifecycle)).

**Reduced motion:** Direct manipulation remains immediate; timed `<animation-list>` `android:tickMark` drawables freeze strictly on frame `0` with zero wakeup.

## Acceptance criteria

- Import integer min, max, and progress values and clamp progress.
- Drag to either endpoint and verify persistent state.
- Verify primary, buffer, track and thumb tints independently, including pressed and disabled color selectors.
- Verify range endpoints and intermediate tick marks, pressed tint and thumb overlap.
- Check state-selected tick shapes, timed `<animation-list>` tick marks (`RULE_COMPOUND_ANIMATION_LIST = 295`, looping/`oneshot`, clip-gated wakeups, reduced-motion freeze), and disabled marks.
- Compare split and continuous tracks using a transparent thumb, including a drag and RTL.
- Check custom thumb shape, pressed and disabled selectors, and split gap sizing.
- Compare appearance and motion with Android on a device.

## Validation and visual status

Importer tests ([`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), `248` passing tests, `0` diffs across `167` existing layouts, including Wave 21B `test_wave21b_compound_animation_list_rule_295` verifying `RULE_COMPOUND_ANIMATION_LIST = 295` `"tick:..."` emission + frame-0 static fallback `kind: 155` + `<inset>` unwrapping/validation) and Rust rendering tests ([`tests/android_xml.rs`](../../tests/android_xml.rs), `320` tests within `915` default / `921` all-features Rust tests across `76` suites, including Wave 21A `wave21a_compound_animation_list_all_slots_rtl_clip_and_reduced_motion` verifying `oneshot` 5-tick playback, final-frame clamp with zero wakeup, and reduced-motion freeze) check independent tint layers, tick count, spacing and draw order, custom thumb selectors, split-track clipping and drag updates. Focused webxdc examples expose these behaviors for manual review. Android device appearance and motion parity have not been established; the track and default thumb use Bhoo's slider geometry.

## Guidelines and visual references

- [Android SeekBar guide](https://developer.android.com/reference/android/widget/SeekBar); no verified side-by-side visual snapshot is available for this change.

## Downloaded reference implementations

- Android 15 [AbsSeekBar.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/AbsSeekBar.java), local `context/reference-sources/android-framework/AbsSeekBar.java` (Apache-2.0), for thumb tint and state handling.
- Android 15 [ProgressBar.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/ProgressBar.java), local `context/reference-sources/android-framework/ProgressBar.java` (Apache-2.0), for inherited progress tint attributes. Bhoo's paint and selection logic are independent implementations.

## References

- [Android SeekBar reference](https://developer.android.com/reference/android/widget/SeekBar)
