# Android XML switch

Status: Partial Android XML `Switch` / `SwitchCompat` / `SwitchMaterial` / `MaterialSwitch` support with Wave 6B Material 3 (`52 × 32dp` pill track, `2dp` unchecked outline, `16dp`/`24dp`/`28dp` thumb) and AOSP (`34 × 14dp` track, `20dp` elevated thumb) density-scaled visual parity under non-`Bhoo` `AndroidTheme` presets; side-by-side Android device pixel and motion parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (`Kind::Switch`) |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

A binary Android XML `Switch` (plus `androidx.appcompat.widget.SwitchCompat`, `com.google.android.material.switchmaterial.SwitchMaterial`, and `com.google.android.material.materialswitch.MaterialSwitch` tag aliases in `tools/import-android-layout.py`) with on, off and disabled states.

## State and interaction contract

The XML `android:checked` value supplies the initial state. Runtime state is stored by view ID and toggles on activation; disabled switches do not toggle. The control follows the label and its thumb direction mirrors in RTL. Explicit `android:textOn` and `textOff` labels are displayed inside the thumb unless `android:showText="false"`; `showText="true"` uses the imported labels and the current ON/OFF defaults. The thumb and track widen to accommodate the longer label. During animation the label follows the presented thumb position and switches at the midpoint. Without explicit labels or `showText`, existing Bhoo examples retain their text-free appearance because full Android theme defaults are not resolved.

State text uses Bhoo's fixed 9 sp style (scaled via `state.sp(9.0)` under non-`Bhoo` Android themes) and clipped thumb bounds; Android switch text appearances, thumb checkmark/close icons (`thumbIcon`), custom drawables and full theme resolution are not yet supported.

The external label supports XML text size, bold, all-caps, letter spacing, stateful text color and the supported local `textAppearance` subset. It uses Bhoo's single-line control layout. The separate on/off text inside the thumb retains its own fixed style.

`android:thumbTint` and `android:trackTint` accept literal colors, color resources and supported checked, enabled, pressed, focused, selected, activated and hovered selectors. They color the thumb and track independently (and override the active `AndroidTheme` defaults below). The track tint colors the track uniformly; Android drawable alpha composition, custom drawables, inherited `buttonTint`, tint blend modes and full theme resolution are not implemented.

### Wave 6B `AndroidTheme` visual parity

`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` renders theme-matched switch geometry controlled by `LayoutState.theme_style: AndroidTheme` and scaled by `LayoutState.effective_density()`:

| `AndroidTheme` | Track geometry & fill | Unchecked border | Thumb geometry & fill | State layer |
| --- | --- | --- | --- | --- |
| `Bhoo` (default) | Native Bhoo `switch_trailing_tinted` control unchanged | Native Bhoo default | Native Bhoo default | Native Bhoo default |
| `Material3Light` / `Material3Dark` | `52 × 32dp` pill (`16dp` radius), `surfaceContainerHighest` (`#E6E0E9` light) -> `primary` (`#6750A4` light); disabled `background.mix(text, 0.12)` | `2dp` `outline` (`#79747E` light) fading out as track checks (`text` at `0.12` alpha when disabled) | `16dp` unchecked `outline` -> `24dp` checked `onPrimary` (`#FFFFFF` light), expanding to `28dp` while pressed (`0.1s` linear in / `0.25s` standard out); widens for `showText` labels | `40dp` circular halo centered on thumb |
| `MaterialLight` / `MaterialDark` | AOSP `34 × 14dp` pill (`7dp` radius), `text` at `0.30` alpha -> `primary` at `0.30` alpha (`0.12` disabled) | None | `20dp` circular thumb with `1.5dp` density-scaled elevation shadow when enabled (`#ECECEC` light / `#BDBDBD` dark -> `primary`) | `40dp` circular halo centered on thumb |

## Motion and animation

Bhoo's shared geometry animation (`animate_geometry`) moves the thumb and interpolates track/thumb colors between states; under `Material3Light`/`Material3Dark`, a press-scale tween (`0.1s` linear in, `0.25s` standard out) expands the thumb diameter to `28dp`. See [Motion and Animation Design](../motion.md#motion-profiles).

**Reduced motion:** State changes are presented immediately.

## Acceptance criteria

- Import checked, unchecked and disabled XML values.
- Activate repeatedly and verify state persists across frames.
- Check separate thumb and track colors in checked, unchecked, pressed and disabled states.
- Check Material 3 (`52 × 32dp` track, `2dp` unchecked outline, `16dp`/`24dp` thumb) and AOSP (`34 × 14dp` track, `20dp` elevated thumb) geometry at `1.0×` and `1.5×` density.
- Check on/off state labels, longer labels, `showText="false"`, motion and RTL placement.
- Compare appearance and animation with Android on a device.

## Validation and visual status

- **Rust tests (`codes/libs/ui/libbhoo/tests/android_xml.rs`, `254` tests within `826` Rust tests across `72` binaries):** `material3_and_aosp_form_control_visual_parity_switch_checkbox_radio_spinner` verifies `Material3Light` unchecked `52 × 32dp` `surfaceContainerHighest` track + `2dp` `outline` border + `16dp` thumb, checked `52 × 32dp` `primary` track (no outline) + `24dp` `onPrimary` thumb, and `density = 1.5` scaling (`78 × 48px` track, `36 × 36px` checked thumb). Existing tests (`switch_state_text_follows_toggle_and_fits_wide_labels`, `switch_thumb_and_track_tints_follow_checked_pressed_and_disabled`, `switch_xml_layout_dimensions_apply_in_both_directions`, `switch_thumb_text_padding_expands_the_track`, `switch_runtime_label_remeasures_and_preserves_checked_state`) verify tints, selectors, state text, sizing, and runtime label updates.
- **Native GLES showcase & webxdc:** Checked and unchecked `Material3Light` switches are rendered in the Wave 6B headless Weston + GLES showcase [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) (ignored local capture `context/android-xml-comparison/bhoo-m3-showcase.png`, `720 × 1280` @ `density = 1.5`). Focused webxdc examples expose on, off, disabled, and RTL states. Side-by-side Android device pixel and motion parity remain unverified.

## Guidelines and visual references

- [Android Switch reference](https://developer.android.com/reference/android/widget/Switch)
- [Material 3 Switch guidelines](https://m3.material.io/components/switch/overview)
- Local ignored Wave 6B showcase capture `context/android-xml-comparison/bhoo-m3-showcase.png` includes checked/unchecked `Material3Light` switches (not a side-by-side Android device capture or bundled asset).

## Downloaded reference implementations

- Android 15 [Switch.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/Switch.java), local `context/reference-sources/android-framework/Switch.java` (Apache-2.0), for separate thumb and track tint attributes and drawable states. Bhoo renders its own switch independently.
- Material Components for Android [`materialswitch/res/values/dimens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/materialswitch/res/values/dimens.xml) and [`materialswitch/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/materialswitch/res/values/tokens.xml) at commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, local `context/reference-repos/material-android/lib/java/com/google/android/material/materialswitch/res/values/` (Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE)), consulted for Material 3 `52 × 32dp` track and `16dp`/`24dp`/`28dp` thumb tokens. See [licenses and attribution](../licenses-and-attribution.md).

## References

- [Android Switch reference](https://developer.android.com/reference/android/widget/Switch)

## XML switch sizing and spacing

`switchMinWidth`, `switchPadding`, and `thumbTextPadding` accept nonnegative dp values up to 4096. They set the minimum track width, gap between the external label and track, and padding around the on/off thumb text. A wrap-content switch measures the label and the configured track and gap. The track stays at least Bhoo's 52 dp default, and it is constrained by the available view width; Android theme defaults, custom track/thumb drawables and optical insets are not reproduced. Android device parity remains unverified. Android 15 Switch.java above describes the corresponding measurement and layout behavior.

The width measurement uses the XML external label when a wrap-content switch is laid out in vertical linear and frame-style containers; the control also measures on the cross-axis in scroll containers. Regression coverage checks the actual 96 dp track and RTL placement. Other Android measurement details and containers remain under review.

## Runtime label updates

Setting `LayoutState.text` for a Switch ID replaces its external XML label at render time; removing the entry restores the XML label. A wrap-content switch remeasures the displayed label without resetting its checked state. Label case and typography use the existing XML text attributes. This supports Bhoo host state updates; Android TextView span support and full TextView measurement remain outside this subset.
