# Chip

Status: Partial implementation — filter-chip motion primitive, Wave 14B 4-variant native `Frame::chip_with_options` (`ChipKind`, `ChipOptions`, elevation, close click), and Android XML `Chip` (`RULE_CHIP_STYLE = 230`) + Wave 14C/15A wrapping `ChipGroup` (`RULE_CHIP_GROUP_WRAP = 269`); leading avatars/icons, keyed removal reflow, and device visual parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Layout](../motion.md#motion-profiles) |

## Scope and variants

Assist, filter, input, suggestion; selected/removable where applicable.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Distinguish assist, filter, input and suggestion behavior; removable chips expose a separately named removal action.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

The current filter primitive follows the pinned Angular Material 22.2.6
`chip.scss` selection transition: leading graphic width and checkmark translation
use 150 ms cubic-bezier(0.4, 0, 0.2, 1); the checkmark path draws over 150 ms
after a 45 ms delay. Label padding moves from 12 to 36 px. Selection colors
and semantic state commit immediately; deselection hides the tick immediately
while the label moves back. Hover/focus overlay opacity uses 150 ms linear
transitions. Interrupted transitions preserve their presentation and apply CSS
reversal shortening, rather than restarting a full-duration spring.

The graphic clips the vector checkmark, using the upstream 20 px SVG geometry
and viewBox (-2, -3, 30, 30). The selected container has no outline. Animation
properties use the shared bounded motion cache; no per-chip timer is added.
The two-segment renderer currently uses its standard round line caps/joins,
so this timing validation is not a claim of identical SVG edge rasterization.

`node tools/test-bhoo-chip-motion.mjs` seeks actual Angular CSS transitions and
samples the compiled Rust/WASM at the same elapsed times. It checks selection,
deselection, the delayed path reveal and interrupted reversal. Evidence is saved
to ignored `context/bhoo-chip-motion.json`. Rust tests also check initial selected
state, immediate logical commits, reduced motion and return to idle.

Removal and keyed reflow remain proposed work: collapse a removed item only
after reassigning focus. Distinct assist/input/suggestion/removal behavior is not
implemented by this filter-chip primitive.

**Reduced motion:** Apply final geometry immediately and retain the same stable focus/selection identity; no sliding or size tween.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Remove the focused chip from an RTL wrap layout and verify deterministic focus recovery.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/chip.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/chips)

- [Local upstream visual example](../material-design/upstream/docs/components/images/chips/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [filter-chip.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/chips/internal/filter-chip.ts) | [Downloaded file](../../../../context/reference-sources/material-web/chips/internal/filter-chip.ts) |
| Material Web | [chip.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/chips/internal/chip.ts) | [Downloaded file](../../../../context/reference-sources/material-web/chips/internal/chip.ts) |
| Material Components for Android | [Chip.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/chip/Chip.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/chip/Chip.java) |
| Compose Material 3 | [Chip.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Chip.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Chip.kt) |
| Angular Material | [chip.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/chips/chip.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/chips/chip.ts) |
| Flutter material_ui | [chip.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/chip.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/chip.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: chip.md](../material-design/upstream/docs/components/chip.md)
- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Native 4-variant chips and Android XML `ChipGroup` support (Wave 14B / Wave 14C / Wave 15A / Wave 17A / Wave 17B)

- **Native M3 `Frame::chip_with_options` (Wave 14B, [`widgets.rs`](../../src/systems/material/mod.rs), `ChipKind` & `ChipOptions` re-exported at crate root in [`lib.rs`](../../src/lib.rs)):**
  - `ChipKind::{Assist, Filter, Input, Suggestion}` and `ChipOptions { kind, selected, elevated, show_close, enabled }` (`32dp` visual height centered inside a `48dp` touch target, `8dp` corner radius).
  - `Assist` is never checkable (ignores `selected = true`); `Filter` draws `secondary_container` fill + leading checkmark when selected; `Suggestion` draws `secondary_container` fill without checkmark when selected; `elevated = true` uses `surface_low` + elevation shadows; `show_close = true` draws a trailing crossed `X` and returns `(Response, bool)` separating body activation from close-button clicks.
- **Android XML `Chip` (`RULE_CHIP_STYLE = 230`), `ShapeAppearance` cut corners (`RULE_CORNER_FAMILY = 278`) & `ChipGroup` (`RULE_CHIP_GROUP_WRAP = 269`) (Wave 4A/4B, Wave 14C/15A & Wave 17A/17B, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - `Chip` renders chamfered (`cut`) and mixed `ShapeAppearance` corners (`RULE_CORNER_FAMILY = 278`) using anti-aliased cached `cut_shape_images` fills and miter-inset `DrawCommand::Line`/`Arc` outlines (Wave 17A/17B).
  - `ChipGroup` emits `RULE_CHIP_GROUP_WRAP = 269` (`"<single_line:0|1>|<single_selection:0|1>|<h_spacing_dp>|<v_spacing_dp>"`). At runtime on `Kind::Linear`, multiline `ChipGroup`s wrap child chips across rows using density-scaled `h_spacing`/`v_spacing`, report accurate multi-row `Size::Wrap` height so following siblings sit cleanly below, and enforce single-selection exclusion across sibling filter chips.
- **Validation:**
  - Wave 14B Rust test: `chip_with_options_supports_all_four_kinds_elevation_and_close_button_click` in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs).
  - Wave 15A/14C & Wave 17A/17B tests: `wave15a_chip_group_wrap_multiline_and_single_selection` and `wave17a_cut_corners_shape_appearance_fill_outline_and_widgets` in [`tests/android_xml.rs`](../../tests/android_xml.rs), plus `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269` and `test_wave17b_cut_corner_family_rule_278` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
  - Static Light/Dark Mesa GLES Page-3 captures: `context/android-xml-comparison/bhoo-m3-showcase-page3.png` and `bhoo-m3-showcase-page3-dark.png`.
- **Remaining limitations:** Leading custom avatars/icons in native `chip_with_options`, animated keyed removal reflow, horizontal scroll on `singleLine="true"` XML `ChipGroup`, and Android device visual parity remain open. See [Android XML import](../android-xml.md).
