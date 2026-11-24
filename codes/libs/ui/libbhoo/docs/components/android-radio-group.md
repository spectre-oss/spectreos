# Android XML radio group

Status: Partial Android XML `RadioGroup` and `RadioButton` / `AppCompatRadioButton` / `MaterialRadioButton` support with single group selection and Wave 6B density-scaled Material 3 / AOSP `20 × 20dp` ring (`2dp` stroke) + `10dp` inner dot visual parity under non-`Bhoo` `AndroidTheme` presets; Android device pixel and motion parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (`Kind::RadioGroup` / `Kind::RadioButton`) |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Android XML `RadioGroup` and `RadioButton` (plus `androidx.appcompat.widget.AppCompatRadioButton` and `com.google.android.material.radiobutton.MaterialRadioButton` tag aliases in `tools/import-android-layout.py`) with selected, unselected and disabled options.

## State and interaction contract

The group retains a single checked direct child and changes it when an enabled choice is activated. The initial choice can be specified with `android:checked` on a child or `android:checkedButton` on the group. The initial `checkedButton` reference may point to a uniquely identified RadioButton descendant. As in Android, selection tracking is attached only to direct RadioButton children: choosing a direct child clears an initially checked nested reference, but a later tap on the nested button is independent and can leave both checked. Duplicate referenced IDs are rejected. The host can request selection by named descendant ID or clear it with `LayoutState::set_radio_group_checked`; requests apply on the next group render. Like Android `check`, an unknown ID becomes the group selection without checking a view. Replacing or clearing a selection unchecks the previous referenced RadioButton. The host can drain selection notifications with `LayoutState::take_radio_group_events`; each entry is a Bhoo view ID or `None` for clear. Changes through direct-child input and processed host requests notify; selecting the same non-empty ID does not, while a clear request notifies even if already clear, as in Android. Initial XML selection is not queued. Requests and notifications are processed during rendering rather than synchronously, and only the latest 32 events per group are retained. Nested RadioGroup interactions and full Android lifecycle behavior remain incomplete.

Indicators and labels follow the layout direction.

RadioButton labels support XML text size, bold, all-caps, letter spacing, stateful text color and the supported local `textAppearance` subset. Labels use Bhoo's single-line control layout rather than full Android text measurement.

RadioButton accepts `android:buttonTint` as a literal color, color resource or supported state selector, including checked, pressed and enabled states. It colors the radio ring and selected dot; the group still controls which child is checked.

### Wave 6B `AndroidTheme` visual parity

Under non-`Bhoo` Android themes (`Material3Light`, `Material3Dark`, `MaterialLight`, `MaterialDark`), `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` renders a density-scaled Material / AOSP radio indicator (`d = state.effective_density()`, preserving Bhoo's native `radio_directional_tinted` control unchanged under `AndroidTheme::Bhoo`):

- **Indicator geometry:** `20 × 20dp` circular outer ring (`10dp` radius, `2dp` stroke width, centered vertically and inset `10dp` from the leading edge in LTR or trailing edge in RTL), `10 × 10dp` circular inner dot (`5dp` radius) when selected, `40dp` circular state-layer halo, and `44dp` indicator-to-label offset.
- **Colors:** Ring transitions between `t.on_surface_variant` (`#49454F` in `Material3Light`) unselected and `t.primary` (`#6750A4` in `Material3Light`) selected (or `buttonTint` when set; `t.text` at `0.38` alpha disabled); inner dot renders in `buttonTint` or `t.primary` (`t.text` at `0.38` alpha disabled).

`android:button` accepts `@null`, a solid XML shape, a selector of supported shapes and `@null`, or (Wave 21A/21B) a timed `<animation-list>` drawable (including `<inset>`-wrapped lists with cyclic-reference detection, emitting `RULE_COMPOUND_ANIMATION_LIST = 295` `"button:<oneshot:0|1>|<img_key_0>:<dur_ms_0>,..."` alongside frame-0 static fallback `kind: 165` and playing frames via `select_animation_list_frame` using `f.time()` and clip-gated `f.wake_at()`). Shapes need explicit dimensions up to 32 dp and support rectangles, uniform rounded corners and circular ovals. Selectors support enabled, pressed, focused, selected, activated, hovered and checked states. Custom shapes replace the built-in ring/dot and can be tinted with the default source-in behavior, multiplying drawable and tint alpha; Bhoo positions them in its existing 40 dp indicator slot (density-scaled under non-`Bhoo` Android themes). Static bitmap/vector `android:button` drawables outside `<animation-list>`, `<animated-vector>`/`<animated-selector>` drawables, tint blend modes beyond src_in, src_atop, src_over, multiply, screen and add, antialiased edge compositing and exact intrinsic padding remain unsupported.

Explicit `android:buttonTintMode` accepts `src_in`, `src_atop`, `src_over`, `multiply`, `screen` and `add` for supported custom button shapes. The importer rejects this attribute for the built-in indicator until its underlying drawable tinting is supported.

## Motion and animation

Under `AndroidTheme::Bhoo`, Bhoo's radio control animates its built-in selection indicator. Under non-`Bhoo` Android themes, `50ms` linear ring color and dot opacity tweens pair with a `300ms` `EmphasizedDecelerate` inner-dot scale tween. Custom solid-shape selectors change immediately; timed `<animation-list>` `android:button` drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`) advance by frame duration using `f.time()` and clip-gated `f.wake_at()`, while `<animated-vector>`/`<animated-selector>` drawables remain unsupported. See [Motion and Animation Design](../motion.md#motion-profiles).

**Reduced motion:** Selection changes are presented immediately; timed `<animation-list>` `android:button` drawables freeze strictly on frame `0` with zero wakeup.

## Acceptance criteria

- Import radio groups and child checked values.
- Switch between enabled options and reject disabled selection.
- Verify the selected, unselected, pressed and disabled indicator colors and single selection.
- Verify Material 3 / AOSP `20 × 20dp` ring (`2dp` stroke) and `10dp` selected dot geometry at `1.0×` and `1.5×` density.
- Verify custom drawable selector shapes, disabled behavior and group exclusivity in both directions.
- Compare appearance and animation with Android on a device.

## Validation and visual status

- **Rust tests (`codes/libs/ui/libbhoo/tests/android_xml.rs`, `320` tests within `915` default / `921` all-features Rust tests across `76` suites):** Wave 21A `wave21a_compound_animation_list_all_slots_rtl_clip_and_reduced_motion` verifies timed `<animation-list>` `android:button` playback (`RULE_COMPOUND_ANIMATION_LIST = 295`) on `Kind::RadioButton` alongside `CheckBox`, clip-gated wakeups, and reduced-motion freeze. `material3_and_aosp_form_control_visual_parity_switch_checkbox_radio_spinner` verifies `Material3Light` unchecked `20 × 20dp` `2dp`-stroke `onSurfaceVariant` ring (no inner dot), checked `20 × 20dp` `primary` ring + `10 × 10dp` `primary` inner dot, and `density = 1.5` scaling (`30 × 30px` ring, `15 × 15px` dot). Existing tests (`radio_group_selects_one_choice_and_rejects_disabled_choice`, `radio_group_checked_button_sets_initial_selection`, `radiobutton_button_tint_tracks_checked_pressed_and_disabled_state`, `radio_custom_button_tint_preserves_alpha_and_selection`, `radio_group_custom_button_selector_preserves_single_selection_and_rtl`, `checkbox_and_radio_mirror_indicator_and_label_in_rtl`, `radio_runtime_label_remeasures_and_preserves_group_selection`) verify tint selectors, custom drawables, RTL, and group selection.
- **Python XML importer tests (`tools/test-android-layout.py`, `248` passing tests, `0` diffs across `167` existing layouts):** Wave 21B `test_wave21b_compound_animation_list_rule_295` verifies `<inset>`-wrapped `<animation-list>` `android:button` emission (`RULE_COMPOUND_ANIMATION_LIST = 295` `"button:..."`) alongside frame-0 static fallback `kind: 165` on `RadioButton`.
- **Native GLES showcase & webxdc:** A horizontal `Material3Light` `RadioGroup` (`Light` selected, `Dark` unselected) is rendered in the Wave 6B headless Weston + GLES showcase [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) (ignored local capture `context/android-xml-comparison/bhoo-m3-showcase.png`, `720 × 1280` @ `density = 1.5`). Focused webxdc examples expose separate tinted and drawable groups, a disabled option, and RTL. Pixel and animation parity with an Android device have not been established.

## Guidelines and visual references

- [Android RadioButton reference](https://developer.android.com/reference/android/widget/RadioButton)
- [Material 3 Radio button guidelines](https://m3.material.io/components/radio-button/overview)
- Local ignored Wave 6B showcase capture `context/android-xml-comparison/bhoo-m3-showcase.png` includes selected/unselected `Material3Light` radio buttons (not a side-by-side Android device capture or bundled asset).

## Downloaded reference implementations

- Android 15 [CompoundButton.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/CompoundButton.java), local `context/reference-sources/android-framework/CompoundButton.java` (Apache-2.0), describes inherited button tint and states.
- Android 15 [RadioGroup.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/RadioGroup.java), local `context/reference-sources/android-framework/RadioGroup.java` (Apache-2.0), describes group selection. Bhoo colors its own indicator independently.
- Material Components for Android [`radiobutton/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/radiobutton/res/values/tokens.xml) at commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, local `context/reference-repos/material-android/lib/java/com/google/android/material/radiobutton/res/values/` (Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE)), consulted for Material 3 `20dp` outer ring, `10dp` dot, and `40dp` state-layer tokens. See [licenses and attribution](../licenses-and-attribution.md).

## References

- [Pinned AOSP RadioGroup source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/RadioGroup.java)
- [Android RadioGroup reference](https://developer.android.com/reference/android/widget/RadioGroup)
- [Android RadioButton reference](https://developer.android.com/reference/android/widget/RadioButton)

A dedicated Android XML example covers a nested initial reference and subsequent direct-child tracking. Importer and Rust tests cover nested initialization, direct checked-child precedence, independent nested interaction, programmatic selection, clearing and unknown IDs; Phone-sized browser tests check initial selection, direct and nested taps, programmatic selection, clearing, reset and RTL. Android device behavior and appearance remain unverified. The pinned Apache-2.0 [Android 15 RadioGroup source](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/RadioGroup.java) is available at `context/reference-sources/android-framework/RadioGroup.java`.

RadioGroup event tests cover direct and independent nested taps, programmatic selection, repeated selection and clear, unknown IDs and bounded retention. The example displays the latest event; a 320 × 720 browser test checks the displayed notifications in LTR and RTL. Android device callback timing is not yet verified.

## Runtime RadioButton labels

A host may override a RadioButton label via `LayoutState.text` keyed by its view ID; removing the entry restores the XML label. Supported linear and frame-style layout paths remeasure a wrap-content radio when its label changes, preserving its group selection. Bhoo reserves its own 48 dp indicator slot and uses its text metrics. Android drawable intrinsic padding, full TextView layout and visual parity remain unverified. The pinned Android CompoundButton and RadioGroup sources above are Apache-2.0.
