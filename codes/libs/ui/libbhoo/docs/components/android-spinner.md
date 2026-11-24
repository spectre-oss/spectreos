# Android XML Spinner

Status: Partial dropdown and dialog Android XML `Spinner` / `AppCompatSpinner` support with string-array or host-provided entries and Wave 6B Material 3 (`4dp` outlined box) vs. AOSP (bottom underline) density-scaled visual parity under non-`Bhoo` `AndroidTheme` presets; side-by-side Android device pixel and popup motion parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (`Kind::Spinner`) |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

An Android `Spinner` (plus `androidx.appcompat.widget.AppCompatSpinner` tag alias in `tools/import-android-layout.py`) displays one selected adapter item and opens a dropdown or single-choice dialog. The XML importer accepts `android:entries` string arrays, `android:spinnerMode`, `android:prompt`, dropdown width, and horizontal/vertical offsets; the host can supply entries in `LayoutState.spinner_items`. Selection is stored in `LayoutState.spinner_selected`. Without a specified mode Bhoo currently chooses dropdown; Android's effective default depends on its theme.

## State and interaction contract

A click opens the dropdown; clicking an item commits the selection and reports the XML ID. Arrow keys move the highlight, Enter commits, and Escape or an outside press closes without committing. A disabled spinner cannot open. The host may set individual item enabled states through `LayoutState.spinner_enabled`; unavailable items are dimmed, cannot be selected and are skipped by keyboard navigation. `LayoutState::set_spinner_adapter` validates updates and preserves selected and highlighted items by stable ID when old and new IDs are provided; otherwise it retains the nearest valid position. The host can also provide separate static, noninteractive XML templates for the selected item and popup rows, with one TextView bound to each entry. Popup XML rows can wrap at the available width; selected XML views can also change height when the Spinner uses `wrap_content`. Interactive row children and full adapter view behavior are not yet supported. An empty adapter clears selection and closes the popup. The dropdown fills the available space, chooses the side with available viewport space, aligns in RTL, closes if its anchor leaves the viewport, and supports touch dragging, fling, wheel scrolling and keyboard navigation. Wrap-content popup width measures up to 15 selected XML item views, as Android does; intrinsic text width remains an approximation of Android measurement. Full Android dropdown/dialog platform appearance parity remains open.

### Wave 6B `AndroidTheme` visual parity

When a `Spinner` omits an explicit background, tint, stroke, gradient, layer-list, or stateful layer, `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` paints theme-matched closed-control chrome scaled by `LayoutState.effective_density()`:

| `AndroidTheme` | Container chrome | Label padding | Trailing / RTL leading dropdown arrow |
| --- | --- | --- | --- |
| `Bhoo` (default) | Legacy `4px`-radius `surface_highest` filled box | `12px` | Unscaled `2px` `on_surface_variant` chevron (`40px` arrow reserve) |
| `MaterialLight` / `MaterialDark` | AOSP bottom underline (`1dp` `#6B000000` light / `#8AFFFFFF` dark unfocused -> `2dp` `t.primary` when focused or open; `0.38` alpha disabled; inset `4dp` horizontally), no full box outline | `12dp` | Density-scaled `2dp` `on_surface_variant` chevron (`0.38` alpha disabled; `40dp` arrow reserve, mirrored in RTL) |
| `Material3Light` / `Material3Dark` | Material 3 `4dp`-radius outlined box (`DrawCommand::Outline` `1dp` `t.outline` `#79747E` light unfocused -> `2dp` `t.primary` when focused or open; `t.text` at `0.12` alpha disabled), no legacy filled surface box | `16dp` | Density-scaled `2dp` chevron (`on_surface_variant`, switching to `t.primary` while open; `0.38` alpha disabled; `40dp` arrow reserve, mirrored in RTL) |

## Motion and animation

The dropdown opens and closes immediately. Dialog mode uses Bhoo's dialog transition. Android's exact popup and dialog motion remain to be validated. See [Motion and Animation Design](../motion.md#motion-profiles).

**Reduced motion:** The dropdown opens and closes immediately.

## Acceptance criteria

- Import string-array entries and select an item.
- Dismiss without changing the selection, and commit keyboard selection.
- Verify `Material3Light`/`Material3Dark` `4dp` outlined container and `MaterialLight`/`MaterialDark` bottom underline plus trailing/RTL dropdown chevron.
- Validate dropdown placement, scrolling, appearance and motion against Android on a device.

## Validation and visual status

- **Rust tests (`codes/libs/ui/libbhoo/tests/android_xml.rs`, `254` tests within `826` Rust tests across `72` binaries):** `material3_and_aosp_form_control_visual_parity_switch_checkbox_radio_spinner` verifies `MaterialLight` bottom underline (`1dp` `#6B000000`, no box outline) + 3-point dropdown arrow polyline and `Material3Light` `4dp`-radius `1dp` `outline` (`#79747E`) container (without the legacy `surfaceContainerHighest` filled rect) + dropdown arrow. Existing Spinner tests cover dropdown/dialog modes, adapter updates, templates, keyboard/pointer selection, scrolling, and RTL.
- **Native GLES showcase:** A `Material3Light` outlined `Spinner` (`Standard` / `Compact` / `Expanded`) is rendered in the Wave 6B headless Weston + GLES showcase [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) (ignored local capture `context/android-xml-comparison/bhoo-m3-showcase.png`, `720 × 1280` @ `density = 1.5`). Side-by-side Android device pixel and popup motion parity remain open.

## Guidelines and visual references

- [Android Spinner guide](https://developer.android.com/develop/ui/views/components/spinner)
- [Material 3 Menus / Exposed dropdown menu guidelines](https://m3.material.io/components/menus/overview)
- Local ignored Wave 6B showcase capture `context/android-xml-comparison/bhoo-m3-showcase.png` includes an outlined `Material3Light` Spinner (not a side-by-side Android device capture or bundled asset).

## Downloaded reference implementations

- [Android 15 Spinner source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/Spinner.java) (consulted for adapter, dropdown/dialog mode, and wrap measurement behavior).
- Material Components for Android [`textfield/res/values/tokens_dropdown_menu.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/res/values/tokens_dropdown_menu.xml) at commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, local `context/reference-repos/material-android/lib/java/com/google/android/material/textfield/res/values/` (Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE)), consulted for Material 3 outlined dropdown container styling. See [licenses and attribution](../licenses-and-attribution.md).

## References

- [Android 15 Spinner source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/Spinner.java)
- [Android Spinner guide](https://developer.android.com/develop/ui/views/components/spinner)

XML popup rows using `wrap_content` are measured independently at the popup width, including wrapping text and minimum height. Row offsets are cached for scrolling; adapter content, template, width, layout direction, text metrics, and drawable sizes, scales or source overrides invalidate the cache. The focused test page includes a long workspace name and a 64dp minimum-height XML row. `match_parent` popup row height still uses the 48dp fallback, and interactive adapter descendants are not supported.

For a `wrap_content` Spinner, the selected noninteractive XML template is measured using the current item and available width. Changing the selection updates its height; explicit Spinner heights remain fixed. The selected label reserves space for the dropdown arrow in either layout direction.

The focused example includes an XML ImageView in the popup template. Toggle **Large row images** and reopen the list to check that cached row heights follow the updated image scale.
