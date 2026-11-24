# Android XML checkbox

Status: Partial Android XML `CheckBox` / `AppCompatCheckBox` / `MaterialCheckBox` support with binary state, Wave 6B density-scaled Material 3 / AOSP `18 × 18dp` (`2dp` radius and `2dp` stroke) indicator visual parity under non-`Bhoo` `AndroidTheme` presets, and Wave 21A/21B timed `<animation-list>` `android:button` playback (`RULE_COMPOUND_ANIMATION_LIST = 295`); tri-state/error `MaterialCheckBox` modes and Android device pixel/motion parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (`Kind::CheckBox`) |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

A binary Android XML `CheckBox` (plus `androidx.appcompat.widget.AppCompatCheckBox` and `com.google.android.material.checkbox.MaterialCheckBox` tag aliases in `tools/import-android-layout.py`) with checked, unchecked and disabled states.

## State and interaction contract

The XML `android:checked` value supplies the initial state. Runtime state is stored by view ID and toggles on activation; disabled checkboxes do not toggle.

Indicators and labels follow the layout direction.

The XML label supports text size, bold, all-caps, letter spacing, stateful text color and the supported local `textAppearance` subset. The label remains single-line within Bhoo's control layout; full Android text measurement remains open.

`android:buttonTint` accepts literal colors, color resources and supported state selectors including checked, enabled and pressed. Tint applies to the indicator fill and outline; its check stroke remains a contrasting theme color (`t.on_primary` when enabled under non-`Bhoo` Android themes). RadioButton also supports button tint; Switch and ToggleButton inherited button tints are not yet supported.

### Wave 6B `AndroidTheme` visual parity

Under non-`Bhoo` Android themes (`Material3Light`, `Material3Dark`, `MaterialLight`, `MaterialDark`), `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` renders a density-scaled Material / AOSP checkbox indicator (`d = state.effective_density()`, preserving Bhoo's native `checkbox_bool_directional_tinted` control unchanged under `AndroidTheme::Bhoo`):

- **Indicator geometry:** `18 × 18dp` box centered vertically in the view (`x` inset `11dp` from leading edge in LTR or trailing edge in RTL), `2dp` corner radius, and `2dp` stroke width, inside a `40dp` circular state-layer halo with a `44dp` indicator-to-label offset.
- **Colors:** Unchecked outline uses `buttonTint` or `t.on_surface_variant` (`#49454F` in `Material3Light`, `t.text` at `0.38` alpha disabled); checked fill uses `buttonTint` or `t.primary` (`#6750A4` in `Material3Light`, `t.text` at `0.38` alpha disabled); checkmark stroke uses `t.on_primary` (`#FFFFFF` in `Material3Light`, `t.background` disabled) drawn as a 3-point `DrawCommand::Polyline`.

`android:button` accepts `@null`, a solid XML shape drawable, a selector of supported shapes and `@null`, or (Wave 21A/21B) a timed `<animation-list>` drawable (including `<inset>`-wrapped lists with cyclic-reference detection). Shapes require explicit dimensions up to 32 dp and support rectangles, uniform rounded corners and circular ovals. Selectors support enabled, pressed, focused, selected, activated, hovered and checked states. A custom shape replaces the built-in checkbox mark; `buttonTint` applies the default source-in tint to the shape, multiplying drawable and tint alpha. For `<animation-list>`, `tools/import-android-layout.py` (`resolve_animation_list_drawable` + `decode_animation_list_frames`) bundles all frames into `install_drawables`, preserves frame-0 static fallback `kind: 165`, and emits `RULE_COMPOUND_ANIMATION_LIST = 295` (`"button:<oneshot:0|1>|<img_key_0>:<dur_ms_0>,..."`); at runtime (`select_animation_list_frame`), `Kind::CheckBox` renders frames via `f.time()` and clip-gated `f.wake_at()` across both `AndroidTheme::Bhoo` and Material/M3 themes (looping or `oneshot`-clamping, freezing strictly on frame `0` with zero wakeup under `MotionPreference::Reduced`). Bhoo positions custom drawables in its existing 40 dp indicator slot (density-scaled under non-`Bhoo` Android themes). Static bitmap/vector `android:button` drawables outside `<animation-list>`, `<animated-vector>`/`<animated-selector>` button drawables, tint blend modes beyond src_in, src_atop, src_over, multiply, screen and add, antialiased edge compositing and exact intrinsic padding remain unsupported.

Explicit `android:buttonTintMode` accepts `src_in`, `src_atop`, `src_over`, `multiply`, `screen` and `add` for supported custom button shapes. The importer rejects this attribute for the built-in indicator until its underlying drawable tinting is supported.

## Motion and animation

Under `AndroidTheme::Bhoo`, Bhoo's existing checkbox transition animates built-in indicator state changes. Under non-`Bhoo` Android themes, staged CSS-equivalent tweens animate the `18 × 18dp` container fill (`90ms`) followed by the delayed 3-point checkmark polyline stroke (`90ms` delay, `90ms` duration). Custom solid-shape selectors change immediately; timed `<animation-list>` `android:button` drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`) advance by frame duration using `f.time()` and clip-gated `f.wake_at()` wakeups, while `<animated-vector>`/`<animated-selector>` drawables remain unsupported. See [Motion and Animation Design](../motion.md#motion-profiles).

**Reduced motion:** State changes are presented immediately; timed `<animation-list>` `android:button` drawables freeze strictly on frame `0` with zero wakeup.

## Acceptance criteria

- Import checked, unchecked and disabled XML values.
- Activate repeatedly and verify state is retained across frames.
- Check checked, unchecked, pressed and disabled button-tint selectors.
- Check Material 3 / AOSP `18 × 18dp` (`2dp` corner radius, `2dp` stroke) indicator geometry and checkmark polyline at `1.0×` and `1.5×` density.
- Check custom button drawable shapes and timed `<animation-list>` `android:button` drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`) for checked, unchecked, disabled, clip-gated wakeup, reduced-motion, and RTL placement.
- Compare appearance and animation with Android on a device.

## Validation and visual status

- **Rust tests (`codes/libs/ui/libbhoo/tests/android_xml.rs`, `320` tests within `915` default / `921` all-features Rust tests across `76` suites):** Wave 21A `wave21a_compound_animation_list_all_slots_rtl_clip_and_reduced_motion` verifies timed `<animation-list>` `android:button` frame playback (`RULE_COMPOUND_ANIMATION_LIST = 295`), `f.wake_at()` scheduling, offscreen clip wakeup suppression, and reduced-motion frame-0 freeze under `AndroidTheme::Material3Light` (and `RadioButton` under `AndroidTheme::Bhoo`). `material3_and_aosp_form_control_visual_parity_switch_checkbox_radio_spinner` verifies `Material3Light` unchecked `18 × 18dp` `2dp`-radius `2dp`-stroke `onSurfaceVariant` outline, checked `18 × 18dp` `primary` fill + 3-point `#FFFFFF` checkmark polyline, and `density = 1.5` scaling (`27 × 27px` box, `3px` stroke). Existing tests (`checkbox_button_tint_tracks_checked_pressed_and_disabled_state`, `checkbox_custom_button_tint_preserves_drawable_alpha`, `checkbox_custom_button_selector_changes_with_state_and_rtl`, `checkbox_and_radio_mirror_indicator_and_label_in_rtl`, `checkbox_runtime_label_remeasures_without_resetting_state`) verify tint selectors, custom button drawables, RTL, pointer activation, and runtime labels.
- **Python XML importer tests (`tools/test-android-layout.py`, `248` passing tests, `0` diffs across `167` existing layouts):** Wave 21B `test_wave21b_compound_animation_list_rule_295` verifies `RULE_COMPOUND_ANIMATION_LIST = 295` (`"button:..."`) emission alongside frame-0 static fallback `kind: 165` on `CheckBox` and `<inset>`-wrapped `RadioButton`, plus cyclic `<inset>`/`<animation-list>` rejection.
- **Native GLES showcase & webxdc:** Checked and unchecked `Material3Light` checkboxes are rendered in the Wave 6B headless Weston + GLES showcase [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) (ignored local capture `context/android-xml-comparison/bhoo-m3-showcase.png`, `720 × 1280` @ `density = 1.5`). Focused webxdc examples expose selector and custom-drawable states for manual review. The indicator and its transition have not been verified against Android device pixels or motion.

## Guidelines and visual references

- [Android CheckBox reference](https://developer.android.com/reference/android/widget/CheckBox)
- [Material 3 Checkbox guidelines](https://m3.material.io/components/checkbox/overview)
- Local ignored Wave 6B showcase capture `context/android-xml-comparison/bhoo-m3-showcase.png` includes checked/unchecked `Material3Light` checkboxes (not a side-by-side Android device capture or bundled asset).

## Downloaded reference implementations

- Android 15 [CompoundButton.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/CompoundButton.java), local `context/reference-sources/android-framework/CompoundButton.java` (Apache-2.0), for inherited button tint and drawable state. Bhoo applies selector colors independently to its own checkbox indicator.
- Material Components for Android [`checkbox/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/checkbox/res/values/tokens.xml) at commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, local `context/reference-repos/material-android/lib/java/com/google/android/material/checkbox/res/values/` (Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE)), consulted for Material 3 `18dp` container, `2dp` corner/stroke, and `40dp` state-layer tokens. See [licenses and attribution](../licenses-and-attribution.md).

## References

- [Android CheckBox reference](https://developer.android.com/reference/android/widget/CheckBox)

## Runtime label updates

A host may override the external label using `LayoutState.text` keyed by the view ID; removing the entry restores the XML label. In supported linear and frame-style layout paths, a wrap-content checkbox remeasures its label while retaining its checked state. Bhoo reserves its own 48 dp indicator slot and uses its text metrics; Android drawable intrinsic padding, complete TextView measurement, and device visual parity remain unverified. The inherited Android TextView text behavior is represented by the local Apache-2.0 Android source references already recorded for this family.
