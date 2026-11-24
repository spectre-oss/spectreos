# Android XML EditText

Status: Initial XML, browser input adapter, full 32-flag `android:inputType` compilation, `AndroidTheme` default underline / Material 3 container styling, `TextInputLayout` (`RULE_TEXT_INPUT_BOX = 228` + Wave 14C/15A `RULE_TEXT_INPUT_EXT = 268`) `outline`/`filled`/`none` box modes, notched floating label, focus/error stroke, helper/error text, leading `startIcon`, `prefix`/`suffix`, live/overflow character counter, interactive `clear_text`/`password_toggle`/`dropdown_menu` (with `LayoutState.entries` Exposed Dropdown popup), Wave 14A `RULE_TEXT_STYLE_EXT = 257` italic/font-family support, Wave 11B multiline text wrapping, newline preservation, vertical gravity, and inner-bounds clipping (`paint_edit_text`), and Wave 24A/24B non-password auto-link detection, `1dp` underlining, `textColorLink`, and `linksClickable` click handling (`RULE_TEXT_LINKS_SPANS = 306`); IME selection synchronization, interactive multiline caret/scrolling, animated label float, specialized `inputType` input-behavior enforcement, offline `map` address geocoding, arbitrary runtime `SpannableString`/`ClickableSpan` Java code, and device visual parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (`Kind::Edit`; wrapped by `TextInputLayout` on `Kind::Linear` via `RULE_TEXT_INPUT_BOX = 228`) |
| Priority | P1 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

`tools/import-android-layout.py` compiles `<EditText>`, `<androidx.appcompat.widget.AppCompatEditText>`, and `<com.google.android.material.textfield.TextInputEditText>` into `bhoo::android_xml::Kind::Edit` with separate initial `android:text` and `android:hint` values, enabled/disabled state, `android:textSize`, `android:textStyle`, `android:textAllCaps` (`RULE_TEXT_ALL_CAPS = 218`), `android:textAlignment` (`RULE_TEXT_ALIGNMENT = 195`), `android:textColor`, `android:autoLink`/`android:textColorLink`/`android:linksClickable` (`RULE_TEXT_LINKS_SPANS = 306`), and `android:inputType`. `<com.google.android.material.textfield.TextInputLayout>` compiles to `Kind::Linear` with `RULE_TEXT_INPUT_BOX = 228` (and forwards its `android:hint` to a child `EditText` that omits one). `AutoCompleteTextView` (`Kind::AutoComplete`) and `MultiAutoCompleteTextView` (`Kind::MultiAutoComplete`) share the same `inputType` parser, `RULE_TEXT_LINKS_SPANS = 306` non-password auto-link handling, `TextInputLayout` child integration, and `AndroidTheme` field chrome in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`.

## State and interaction contract

The host owns editable text in `LayoutState.text` keyed by view ID (initialized from XML `android:text` when absent), and `Node::render` records each rendered field's compiled input mode in `LayoutState.input_types`. The browser host supplies the native text/IME input adapter and supports multiple independently edited XML fields. Runtime `LayoutState.density`, `LayoutState.font_scale`, and `LayoutState.rtl` scale dimensions, text sizes, paddings, and default text alignment.

### Full 32-flag `android:inputType` support

`tools/import-android-layout.py` (`VALID_INPUT_TYPE_FLAGS`) accepts all 32 standard Android `android:inputType` flags individually or combined with `|` (whitespace around `|` is trimmed) on `EditText`, `AutoCompleteTextView`, and `MultiAutoCompleteTextView`, rejecting empty or unrecognized flags with `ValueError`:

| `Node.input_type` code | Matching `android:inputType` flags (highest precedence first) | Runtime & browser behavior |
| --- | --- | --- |
| `4` (password) | `textPassword`, `textWebPassword`, `numberPassword` | Masks displayed characters with `"•"` in `libbhoo` and requests password input mode in the browser host |
| `1` (email) | `textEmailAddress`, `textWebEmailAddress` | Requests email keyboard mode |
| `2` (phone) | `phone` | Requests phone/dialpad keyboard mode |
| `5` (decimal) | `numberDecimal` | Requests decimal numeric keyboard mode |
| `3` (number) | `number`, `numberSigned` | Requests numeric keyboard mode |
| `0` (text / general) | `none`, `text`, `textCapCharacters`, `textCapWords`, `textCapSentences`, `textAutoCorrect`, `textAutoComplete`, `textMultiLine`, `textImeMultiLine`, `textNoSuggestions`, `textUri`, `textEmailSubject`, `textShortMessage`, `textLongMessage`, `textPersonName`, `textPostalAddress`, `textVisiblePassword`, `textWebEditText`, `textFilter`, `textPhonetic`, `datetime`, `date`, `time` | Standard text input mode |

When multiple flags are combined with `|` (for example `textPassword|textNoSuggestions`, `textWebEmailAddress|textNoSuggestions`, `number|numberDecimal`, or `textCapSentences|textMultiLine`), precedence follows the table order above (`4` -> `1` -> `2` -> `5` -> `3` -> `0`).

### `AndroidTheme` default underline and Material 3 container styling

In `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, single-line `Kind::Edit` / `Kind::AutoComplete` / `Kind::MultiAutoComplete` renders up to `state.dp(48.0)` field height within its resolved inner bounds (while standalone multiline `EditText` uses its full `inner.height`) and adapts its default chrome to `LayoutState.theme_style: AndroidTheme` whenever the view has no explicit `background`, `RULE_BACKGROUND_TINT`, `background_stroke`, `RULE_GRADIENT`, or `RULE_DRAWABLE_LAYER`:

| `AndroidTheme` | Default chrome (when no explicit background/stroke/tint/gradient/layer is set) | Hint and text treatment |
| --- | --- | --- |
| `Bhoo` (default) | Delegates to Bhoo's native dark `Frame::text_field` chrome | Floating label via `text_field`; editable text drawn `16dp` below field top with `12dp` horizontal padding (`0.38` alpha when disabled) |
| `MaterialLight` | Transparent container with a bottom underline inset `4dp` horizontally: `1dp` `#6B000000` unfocused (`0.38` alpha disabled) -> `2dp` `#009688` (`t.primary`) focused | Inline hint in `#61000000` when empty; text in `t.text` (`#DE000000`, or explicit/state `textColor`) with `4dp` horizontal padding |
| `MaterialDark` | Transparent container with a bottom underline inset `4dp` horizontally: `1dp` `#8AFFFFFF` unfocused (`0.38` alpha disabled) -> `2dp` `#80CBC4` (`t.primary`) focused | Inline hint in `#80FFFFFF` when empty; text in `t.text` (`#FFFFFFFF`, or explicit/state `textColor`) with `4dp` horizontal padding |
| `Material3Light` | `4dp`-rounded filled container (`t.surface_highest` `#E6E0E9`, or `t.text` at `0.04` alpha disabled) + `DrawCommand::Outline` (`1dp` `t.outline` `#79747E` -> `2dp` `t.primary` `#6750A4` focused; `t.text` at `0.12` disabled) + full-width bottom active indicator bar (`1dp` `t.on_surface_variant` `#49454F` -> `2dp` `t.primary` focused; `t.text` at `0.38` disabled) | Inline hint in `t.on_surface_variant` when empty and unfocused (`12dp` horizontal padding); floats as a `12sp` label at `y + 4dp` (`t.primary` focused / `t.on_surface_variant` unfocused) when focused or non-empty, with value text offset `16dp` below top |
| `Material3Dark` | Same Material 3 container, `4dp` radius, outline, and bottom indicator using dark tokens (`#36343B` `surface_highest`, `#938F99` `outline`, `#D0BCFF` `primary`, `#CAC4D0` `on_surface_variant`) | Same floating `12sp` hint and `16dp` text offset with dark Material 3 color roles (`#E6E0E9` `t.text`) |

### `TextInputLayout` box modes, floating label, helper/error text, and end icons (`RULE_TEXT_INPUT_BOX = 228`, Wave 4A & 4B)

`tools/import-android-layout.py` lowers `<com.google.android.material.textfield.TextInputLayout>` styles (`OutlinedBox`, `FilledBox`, `ExposedDropdownMenu`, `?attr/textInputOutlinedStyle`, `?attr/textInputFilledStyle`, and local inherited `@style/` definitions) and `TEXT_INPUT_ATTRS` (`app:boxBackgroundMode`, `app:boxCornerRadius*`, `app:boxStrokeColor`, `app:hintEnabled`, `app:helperText`, `app:helperTextEnabled`, `app:errorEnabled`, `app:endIconMode`, `app:passwordToggleEnabled`) into `RelativeRule { kind: RULE_TEXT_INPUT_BOX (228), anchor: "<mode>|<corner>|<stroke_rgba>|<hint>|<helper>|<error_enabled>|<end_icon>" }` (escaping `|` as `/` and newlines as spaces in hint/helper strings, and rejecting unsupported `boxBackgroundMode`, `endIconMode`, or invalid boolean flags with `ValueError`).

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, `Kind::Linear` with `RULE_TEXT_INPUT_BOX = 228` coordinates the container chrome and sets `state.text_input_active_box` while drawing its child `EditText` / `AutoCompleteTextView` / `MultiAutoCompleteTextView` so the child skips duplicate standalone chrome and renders cleanly inside the container's content bounds (`16dp` horizontal padding, RTL-aware alignment):

| Box mode | Container & stroke/indicator chrome | Floating label treatment |
| --- | --- | --- |
| `outline` | Reserves a top half-label inset (`label_h * 0.5`) when a hint is present; paints rounded border (`paint_outline_radii`, default `4dp` corners or `app:boxCornerRadius*`) at `1dp` unfocused (`app:boxStrokeColor` or `t.outline`; `t.text` at `0.12` alpha disabled) -> `2dp` `t.primary` focused (or `t.error` when error active) | Paints a notch cutout (`explicit_background` or `t.background`, `12dp` from leading edge in LTR/RTL) over the top border and renders the `12sp` floating label (`t.error` / `t.primary` / `t.on_surface_variant` / `0.38` disabled) inside the notch |
| `filled` | Paints top-rounded container (`explicit_background` or `t.surface_highest`, default `[4, 4, 0, 0]` dp corners; `t.text` at `0.04` alpha disabled) + full-width bottom active indicator (`1dp` `app:boxStrokeColor` or `t.on_surface_variant` unfocused -> `2dp` `t.primary` focused, or `t.error` on error) | Renders the `12sp` floating label at `box_rect.y + 6dp` (`16dp` leading padding, RTL-aware) and offsets child value text down by `20dp` |
| `none` | Omits container fill and border chrome while preserving child editing, supporting text, and end icons | Uses child field layout without an extra box header |

- **Helper and runtime error text (`<id>:error`):** When `app:errorEnabled="true"` (`error_enabled = 1`) and the host inserts a non-empty error string at `LayoutState.text["<id>:error"]` (keyed by the `TextInputLayout` ID or `#<id>`), the error message replaces `app:helperText` in the supporting-text row below the box (`4dp` top gap, `16dp` horizontal padding, `12sp`) and switches the box stroke/indicator and floating label to `t.error` (`#B3261E` under `Material3Light`). When `error_enabled = 0`, `<id>:error` entries are ignored and `app:helperText` remains displayed in `t.on_surface_variant`.
- **Interactive trailing end icons (RTL-aware `40dp` button target with circular state layer):**
  - `clear_text`: draws `MaterialIcon::Close` (`"Clear text"` target; full `on_surface_variant` when the child field has text, `0.45` alpha when empty, `0.38` disabled); clicking clears `state.text[child_key]` to `""` and returns the `TextInputLayout` ID.
  - `password_toggle` (from `app:endIconMode="password_toggle"` or legacy `app:passwordToggleEnabled="true"`): masks the child field with `"•"` by default, renders a procedural eye outline + pupil with a diagonal slash when masked (and without a slash when revealed; `"Toggle password visibility"` target), and clicking toggles `state.text_input_password_visible[child_key]` to reveal or re-mask the text.
  - `dropdown_menu` (from `app:endIconMode="dropdown_menu"` or `ExposedDropdownMenu` styles): renders a downward chevron polyline (`"Show dropdown menu"` target); clicking toggles an interactive Exposed Dropdown popup backed by `LayoutState.entries` (falling back to `autocomplete_items` / `spinner_items`), selecting a clicked item into `LayoutState.text[child_key]` and returning the `TextInputLayout` ID.
  - (`app:endIconMode="custom"` currently lowers to `none` without rendering custom end-icon drawables.)
- **`TextInputLayout` extended affixes, start icon & live counter (`RULE_TEXT_INPUT_EXT = 268`, Wave 14C / Wave 15A):** `tools/import-android-layout.py` encodes `"<counter_enabled:0|1>|<counter_max>|<prefix>|<suffix>|<start_icon>"` from `app:counterEnabled`, `app:counterMaxLength`, `app:prefixText`, `app:suffixText`, and `app:startIconDrawable`. At runtime, `Kind::Linear` paints the leading `24dp` `start_icon`, inline `prefix` and `suffix` labels, and the trailing `len / max` character counter (switching to M3 `error` `#B3261E` when character count exceeds `counter_max`).

### Multiline text wrapping, newlines, vertical gravity, and clipping (`paint_edit_text`, Wave 11B)

`Node::paint_edit_text` in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` handles text and hint rendering consistently across `TextInputLayout` (`RULE_TEXT_INPUT_BOX = 228`) children and standalone `Kind::Edit` / `Kind::AutoComplete` / `Kind::MultiAutoComplete` under every `AndroidTheme` (`Bhoo`, `MaterialLight`, `MaterialDark`, `Material3Light`, `Material3Dark`):

- **Multiline activation & full height:** Enters multiline rendering when `!single_line && max_lines != 1` and (`min_lines > 1`, displayed text contains `'\n'`, or usable height `>= line_h * 1.9`, excluding `AutoComplete`/`MultiAutoComplete` from the height-only heuristic). Standalone multiline `EditText` keeps its full resolved `inner.height` instead of clamping to `48dp`, while single-line fields keep centered single-line layout (flattening embedded `\n`/`\r` to spaces).
- **Line wrapping & paragraph preservation:** Splits explicit `\n` paragraphs and blank lines cleanly via `wrapped_text_lines` using `resolved_line_height`, `line_spacing_multiplier`, and `line_spacing_extra`, capping painted lines to `min(height_limit, max_lines)` so overflowing lines stop cleanly inside the field.
- **Vertical gravity & inner-bounds clipping:** Supports top (`content_gravity[1] == 0`, `min_lines > 1`, or `text_rect.height >= line_h * 2.2`), bottom (`content_gravity[1] == 2`), and center vertical placement, and pushes/pops a `DrawCommand::PushClip` intersected with `f.clip` inside the field's horizontal/vertical inner bounds so text never bleeds outside the box.

### Auto-link detection, link styling & clicks (`RULE_TEXT_LINKS_SPANS = 306`, Wave 24A / Wave 24B)

- **Importer (`tools/import-android-layout.py`):** Resolves `android:autoLink` (`none`, `web`, `email`, `phone`, `map`, `all`, and `|`-combined flags, `@string/`, direct or via `@style`), `android:textColorLink` (`#hex`, `@color/` including `<selector>` ColorStateList static default, `?attr/textColorLink`, `@null`, direct/`@style`/`textAppearance`), and `android:linksClickable` (`true`/`false`, `@bool/`, default `true`) on `EditText`, `AppCompatEditText`, `TextInputEditText`, `AutoCompleteTextView`, and `MultiAutoCompleteTextView` into `RelativeRule { kind: 306, anchor: "<mask>|<link_rgba>|<clickable:0|1>" }`.
- **Runtime (`paint_edit_text` in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`):** On non-password fields (suppressed whenever `input_type == 4` or `password_toggle` is active so masked `"•"` bullets are never linkified), scans each painted single-line or multiline line for non-overlapping `web`, `email`, and `phone` spans, paints matched spans in `link_color` (explicit RGBA or theme `t.primary` fallback) with a density-scaled `1dp` underline honoring alignment and RTL, and when `clickable == 1` registers `Role::Link` hit targets (`anonymous_child_id(id, 42_000 + link_idx)`) that record the clicked span into `LayoutState.last_clicked_link` and return `Some(self.id)`. Offline `map` address geocoding and arbitrary runtime `SpannableString`/`ClickableSpan` Java code remain outside static XML.

## Motion and animation

Under `AndroidTheme::Bhoo`, standalone `EditText` fields use Bhoo's `Frame::text_field` label and focus state transitions ([Feedback motion profile](../motion.md#motion-profiles)). Under `MaterialLight`, `MaterialDark`, `Material3Light`, `Material3Dark`, and `TextInputLayout` (`RULE_TEXT_INPUT_BOX = 228`), focus/error stroke widths, colors, and floating label placement currently switch directly without animated float interpolation, while `TextInputLayout` trailing end-icon targets use Bhoo's shared circular state layer.

**Reduced motion:** Focus, error, and end-icon state transitions remain immediately usable without decorative motion.

## Performance and lifecycle

Text values live in caller-owned `LayoutState.text` keyed by view ID (or stable anonymous path ID); no persistent animation loop runs when idle.

## Acceptance criteria

- Preserve XML initial text and hint independently across multiple fields.
- Accept all 32 Android `android:inputType` flags and pipe-separated combinations, masking password types (`4`) and rejecting invalid flags explicitly.
- Render theme-appropriate default chrome under `Bhoo`, `MaterialLight`/`MaterialDark` (1dp unfocused -> 2dp primary focused underline), and `Material3Light`/`Material3Dark` (`4dp`-rounded container, outline, bottom indicator, floating `12sp` hint), while respecting explicit XML backgrounds.
- Verify `TextInputLayout` (`RULE_TEXT_INPUT_BOX = 228`) `outline` (notched border) and `filled` modes, focus/error stroke and `<id>:error` supporting text overriding helper text, and interactive `clear_text`, `password_toggle`, and `dropdown_menu` end icons.
- Verify non-password `EditText` `RULE_TEXT_LINKS_SPANS = 306` auto-link highlighting, `1dp` underlining, `Role::Link` clicks updating `LayoutState.last_clicked_link`, and password-field link suppression.
- Edit fields through the browser input adapter and compare rendering and input behavior against Android on a device.

## Guidelines and visual references

- [Android EditText reference](https://developer.android.com/reference/android/widget/EditText)
- [Android InputType reference](https://developer.android.com/reference/android/text/InputType)
- [Android Linkify reference](https://developer.android.com/reference/android/text/util/Linkify)
- [Material 3 Text fields guidelines](https://m3.material.io/components/text-fields/overview)
- Local XML example: [`edittext.xml`](../../../../codes/apps/bhoo-web/android-examples/edittext.xml).
- No verified side-by-side Android device screenshot snapshot is bundled locally for EditText or TextInputLayout; visual parity with Android remains unverified.

## Downloaded reference implementations and attribution

Bhoo's `Kind::Edit` rendering, `TextInputLayout` box chrome (`RULE_TEXT_INPUT_BOX = 228`), and `AndroidTheme` defaults are independently implemented in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` informed by the following pinned Apache-2.0 references:

- Android 15 [TextView.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/TextView.java), [themes_material.xml](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/res/res/values/themes_material.xml), [styles_material.xml](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/res/res/values/styles_material.xml), and [dimens_material.xml](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/res/res/values/dimens_material.xml), local `context/reference-sources/android-framework/` (Apache-2.0): consulted for AOSP `Theme.Material.Light` / `Theme.Material` EditText underline and color tokens, plus pinned upstream AOSP [Linkify.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/text/util/Linkify.java) (Apache-2.0, consulted without a vendored local Java copy) for `autoLink` mask semantics.
- Material Components for Android [`textfield/res/values/styles.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/res/values/styles.xml), [`dimens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/res/values/dimens.xml), and [`tokens_textfield.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/textfield/res/values/tokens_textfield.xml) at commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, local `context/reference-repos/material-android/lib/java/com/google/android/material/textfield/res/values/` (Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE)): consulted for Material 3 `TextInputLayout` outlined/filled box geometry, active indicator, error/helper supporting text, end-icon modes, and floating-label color roles.

See [licenses and attribution](../licenses-and-attribution.md) and [third-party notices](../../../../../../THIRD_PARTY_NOTICES.md).

## Validation evidence

- **Python XML importer tests ([`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), `258` passing tests, `0` diffs across `167` existing layouts):**
  - Wave 24B: `test_wave24b_text_links_spans_rule_306` verifies `RULE_TEXT_LINKS_SPANS = 306` emission across `EditText` and `TextInputEditText` (`autoLink`, `@string/` masks, `textColorLink`, `linksClickable`, `@style`/`textAppearance` inheritance, omission on defaults, and strict error validation).
  - Wave 21B: `test_wave21b_compound_animation_list_rule_295` covers `RULE_COMPOUND_ANIMATION_LIST = 295` timed `<animation-list>` compound drawables across compound-drawable widgets.
  - `test_wave4a_text_input_box_rule_228` and `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269` verify `RULE_TEXT_INPUT_BOX = 228` and `RULE_TEXT_INPUT_EXT = 268` emission across outlined/filled/`ExposedDropdownMenu`/`none` boxes, counter/prefix/suffix/startIcon, custom style inheritance, and `ValueError` rejections.
  - `test_edit_text_separates_hint_and_initial_value` verifies independent `android:text` and `android:hint` compilation.
  - `test_edit_text_input_types` verifies core text, email, phone, numeric, decimal, and password `inputType` lowering.
  - `test_wave3a_full_input_type_flags` verifies all 32 individual Android `android:inputType` flags, combined pipe-separated flag sets (`text|textNoSuggestions`, `textCapSentences|textMultiLine`, `textImeMultiLine|textNoSuggestions|textMultiLine`, `textWebEmailAddress|textNoSuggestions`, `textPassword|textNoSuggestions`, `number|numberDecimal`, `number|numberSigned`, `number|numberPassword`), and explicit `ValueError` rejection of empty/unknown flags.
  - `test_wave3a_text_all_caps_rule_218` verifies `RULE_TEXT_ALL_CAPS = 218` emission via `textAppearance` on `EditText`.
- **Rust runtime and rendering tests ([`codes/libs/ui/libbhoo/tests/android_xml.rs`](../../tests/android_xml.rs), `332` passing tests; `927` default / `933` all-features Rust tests across `76` suites in `libbhoo`):**
  - Wave 24A: `wave24a_text_links_spans_alignment_rtl_justified_clickable_text_and_edit_text` verifies `RULE_TEXT_LINKS_SPANS = 306` link highlighting, `1dp` underlines, `Role::Link` clicks updating `LayoutState.last_clicked_link` on `Kind::Edit`, and suppression on `input_type: 4` password fields.
  - Wave 21A: `wave21a_compound_animation_list_all_slots_rtl_clip_and_reduced_motion` verifies timed `<animation-list>` compound drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`) on `Kind::Edit` in LTR and RTL (with clip-gated `f.wake_at()` and reduced-motion frame-0 freeze).
  - Wave 15A: `wave15a_text_input_ext_affixes_counter_and_exposed_dropdown` verifies leading `24dp` `start_icon`, inline `"$"` prefix and `"USD"` suffix, live `"3 / 8"` counter transitioning to M3 error `#B3261E` on `"10 / 8"` overflow, and clicking the `dropdown_menu` end icon to open and select from `LayoutState.entries`.
  - Wave 14A: `wave14a_text_style_ext_italic_and_font_family_across_widgets` verifies `RULE_TEXT_STYLE_EXT = 257` italic + `FontFamily::Monospace` on `Kind::Edit`.
  - Wave 11B: `multiline_edit_text_and_text_input_layout_wrap_newlines_and_clip_to_bounds` verifies single-line `TextInputLayout` vertical centering (`56dp` box), multiline `TextInputLayout` explicit newline & blank-line paragraph preservation, line wrapping, top alignment, and inner-box `DrawCommand::PushClip`, plus standalone `96dp` multiline `EditText` using full `96dp` height (not clamped to `48dp`), wrapping lines, and clipping overflowing lines inside field bounds.
  - `text_input_layout_outline_and_filled_modes_focus_error_and_end_icons` verifies `outline` mode (`1dp` `#79747E` radius `8dp` border, `#FEF7FF` notch behind `12sp` `"Email"` label, helper text, `MaterialIcon::Close` end icon), focus (`2dp` `#6750A4` border), error activation via `state.text["til_outline:error"]` (`#B3261E` border & error message replacing helper text), clicking `"Clear text"` to clear `state.text["email_edit"]`, `filled` mode (`#E6E0E9` radius `4dp` container, `1dp` `#49454F` -> `2dp` `#6750A4` focused bottom indicator, `error_enabled = 0` ignoring `:error`), and clicking `"Toggle password visibility"` to unmask `"••••••"` to `"secret"` while removing the eye slash.
  - `edit_text_uses_initial_value_hint_and_disabled_state` verifies host-owned initial text, hint, and disabled rendering.
  - `password_edit_text_masks_rendered_value` verifies `input_type: 4` bullet masking and `state.input_types` tracking.
  - `android_theme_edit_text_underline_and_toolbar_surface_elevation` verifies `MaterialLight` unfocused `1dp` `#6B000000` underline -> focused `2dp` `#009688` underline, `MaterialDark` unfocused `1dp` `#8AFFFFFF` underline, and `Material3Light` `4dp`-rounded `#E6E0E9` container + `1dp` `#79747E` outline + `1dp` `#49454F` bottom indicator.
- **Static native GLES captures (Wave 11B & Wave 15A):** `context/android-xml-comparison/gmail-xml-compose.png` (via [`android_gmail_capture.rs`](../../../../codes/libs/ui/libbhoo/examples/android_gmail_capture.rs) `--compose`) and Page-3 Light & Dark captures (`bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png` via [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) `--page3`, rendering `TextInputLayout` with start icon, prefix/suffix, and live counter) under headless Weston + Mesa GLES.

**Visual and device parity status:** Functional state, `inputType` compilation, `TextInputLayout` box/error/end-icon/affix/counter/dropdown behavior, multiline wrapping/clipping, and emitted draw commands are unit-tested in Python and Rust and visually exercised in static Mesa GLES captures; Android device visual/font/IME parity remains unverified.

## Remaining limitations

- **Behavioral `inputType` enforcement:** Flags mapped to code `0` (`textCapCharacters`, `textCapWords`, `textCapSentences`, `textAutoCorrect`, `textMultiLine`, `textImeMultiLine`, `date`, `time`, `datetime`, `textUri`, etc.) compile cleanly without enforcing auto-capitalization, spellcheck suppression, interactive newline/Return editing behavior, or date/time pickers at runtime (multiline *rendering* is handled by `paint_edit_text` above).
- **Selection, caret, and IME:** Grapheme-aware caret/selection synchronization, copy/paste menus, full IME composition spans, and internal vertical scrolling of overflowing multiline text inside `EditText` remain incomplete.
- **Material 3 `TextInputLayout` remaining features:** Animated resting-to-floated label interpolation (hints currently render in their floated position), custom `boxStrokeWidth`/`boxStrokeWidthFocused` widths, custom `endIconDrawable` rendering (`endIconMode="custom"` lowers to `none`), `placeholderText`, and keyboard arrow traversal inside the Exposed Dropdown popup remain open.
- **Auto-links and spans:** Offline `autoLink="map"` address geocoding, arbitrary runtime `SpannableString`/`ClickableSpan` Java code, and stateful `textColorLink` `ColorStateList` pressed/focused transitions remain open.
- **Platform parity:** Android `edit_text_inset_*` 9-patch stretch geometry, font/baseline metrics, and side-by-side device visual/input comparison remain open.
