# Android XML TextSwitcher

Status: Initial XML text switching support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

Android TextSwitcher switches between up to two TextView children when its text changes.

## State and interaction contract

The host can set the current text without switching or set the next text and advance. TextView children may have XML IDs or be anonymous. Anonymous children using wrap-content remeasure after text changes. Nested switchers need their own stable XML ID. View factories and exact Android measurement remain unsupported.

## Motion and animation

Supported XML alpha, translate, Wave 16A/16B `<scale>` and `<rotate>` (`RULE_SCALE_IN = 274`, `RULE_SCALE_OUT = 275`, `RULE_ROTATE_IN = 276`, `RULE_ROTATE_OUT = 277`), and Wave 19A/19B `elevation` / `translationZ` object-animator resources (`RULE_ELEVATION_IN = 288`, `RULE_ELEVATION_OUT = 289`, 7 `AnimInterp` curves, continuous `paint_elevation_dp` interpolation suppressing duplicate static child `RULE_ELEVATION` during active transitions) animate text changes via balanced `DrawCommand::PushTransform` / `PopTransform` calls and interpolated elevation shadows.

**Reduced motion:** Text changes immediately without alpha, translate, scale, or rotate motion, and elevation transitions snap immediately to `to_dp`.

## Acceptance criteria

- Reject non-TextView children and more than two children.
- Set current or next text independently of child IDs.
- Compare text and transition rendering with Android.

## Validation

- Importer tests `test_wave16b_scale_and_rotate_transition_rules_274_to_277` and `test_wave19b_elevation_and_translation_z_animators_rules_288_289` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`241` passing tests) and runtime tests `wave16a_scale_and_rotate_transition_animations_and_reduced_motion` and `wave19a_elevation_object_animator_transitions_and_reduced_motion` in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`312` passing tests; `907` default / `913` all-features Rust tests across `76` suites). Android device visual and timing parity remain unverified.

## References

- [Android TextSwitcher reference](https://developer.android.com/reference/android/widget/TextSwitcher)
