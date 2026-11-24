# Slider

Status: Partial implementation — base bounded pointer/keyboard slider, Wave 14B native M3 Expressive `Frame::expressive_slider` (`SliderExpressiveOptions`), and Wave 14C/15A Android XML `RULE_SLIDER_OPTIONS = 267`; vertical sliders, custom label formatters, and Android device visual parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Direct manipulation](../motion.md#motion-profiles) |

## Scope and variants

Continuous/discrete; ticks, value indicator.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose min/max/value/step; pointer drag updates continuously; arrows and Home/End adjust within limits.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Thumb position follows dragging immediately; only hover/value-bubble appearance is animated.

Use the shared [direct manipulation profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Retain immediate user-controlled movement; disable optional inertial or programmatic travel where practical.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Drag across the full range and verify no lag, invalid value or extra action from the value bubble.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/slider.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/sliders)
- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Slider.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/sliders/)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/slider/hero.webp)
- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/slider/slider-standard.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [slider.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/slider/internal/slider.ts) | [Downloaded file](../../../../context/reference-sources/material-web/slider/internal/slider.ts) |
| Material Components for Android | [Slider.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/Slider.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/slider/Slider.java) |
| Material Components for Android | [BaseSlider.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/BaseSlider.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/slider/BaseSlider.java) |
| Compose Material 3 | [Slider.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt) |
| Angular Material | [slider.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/slider/slider.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/slider/slider.ts) |
| Flutter material_ui | [slider.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/slider.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/slider.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: slider.md](../material-design/upstream/docs/components/slider.md)
- [Material Android: Slider.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Slider.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Native M3 Expressive slider and Android XML support (Wave 14B / Wave 14C / Wave 15A)

- **Native M3 Expressive `Frame::expressive_slider` (Wave 14B, [`widgets.rs`](../../src/systems/material/mod.rs), `SliderExpressiveOptions` re-exported at crate root in [`lib.rs`](../../src/lib.rs)):**
  - `SliderExpressiveOptions { step_size: f32, min_separation: f32, show_value_bubble: bool, enabled: bool }` (defaults: `0.0`, `0.0`, `true`, `true`).
  - Renders `16dp`-tall active (`primary`) and inactive (`secondary_container`) tracks separated by `6dp` gaps on each side of a `4 × 44dp` (`2dp` radius) vertical pill handle, a trailing `4dp` (`2dp` radius) stop indicator dot (`on_secondary_container`), discrete `4dp` tick dots when `step_size > 0.0`, discrete snapping, and a floating teardrop `inverse_surface` (`16dp` radius) / `inverse_on_surface` value bubble above the handle during press or keyboard focus when `show_value_bubble` is enabled.
- **Android XML `Slider` (`RULE_SLIDER_OPTIONS = 267`) (Wave 14C / Wave 15A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - `tools/import-android-layout.py` emits `RULE_SLIDER_OPTIONS = 267` (`"<step_size>|<min_separation>|<label_behavior:floating|withinBounds|gone|visible>|<expressive:0|1>"`) on `com.google.android.material.slider.Slider`. At runtime on `Kind::SeekBar`, enforces discrete `step_size` snapping, Expressive `16dp` track + `4 × 44dp` vertical handle geometry, and `label_behavior` value bubbles (`visible` always shown, `floating`/`withinBounds` on interaction, `gone` suppressed).
- **Validation:**
  - Wave 14B Rust test: `expressive_slider_renders_16dp_tracks_4x44_handle_6dp_gaps_stop_dot_ticks_and_value_bubble` in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs).
  - Wave 15A/14C tests: `wave15a_slider_options_step_min_sep_expressive_and_bubbles` in [`tests/android_xml.rs`](../../tests/android_xml.rs) and `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
  - Static Light/Dark Mesa GLES Page-3 captures: `context/android-xml-comparison/bhoo-m3-showcase-page3.png` and `bhoo-m3-showcase-page3-dark.png`.
- **Remaining limitations:** Vertical slider orientation, custom value-bubble label formatters, animated handle width morph on press, and Android device visual parity remain open. See [Android XML import](../android-xml.md).
