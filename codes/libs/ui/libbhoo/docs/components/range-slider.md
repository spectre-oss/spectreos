# Range slider

Status: Partial implementation — two ordered pointer/keyboard endpoints, Wave 14B native M3 Expressive `Frame::expressive_range_slider` (`SliderExpressiveOptions` with `min_separation`, discrete steps, and value bubbles), and Wave 5A/9B/10A/14C/15A Android XML `RangeSlider` (`RULE_RANGE_SLIDER = 250` + `RULE_SLIDER_OPTIONS = 267`); multi-thumb (`>2`), custom label formatters, and Android device visual parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Direct manipulation](../motion.md#motion-profiles) |

## Scope and variants

Two values and ordered thumbs.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose two ordered endpoints; keyboard users can reach and adjust each thumb without crossing bounds.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Both thumbs track direct manipulation immediately; optional programmatic interpolation must preserve endpoint ordering.

Use the shared [direct manipulation profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Retain immediate user-controlled movement; disable optional inertial or programmatic travel where practical.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Move thumbs toward each other with keyboard and pointer and verify the range never inverts.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Slider.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/sliders/)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider-multithumb/)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/slider/slider-standard.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [slider.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/slider/internal/slider.ts) | [Downloaded file](../../../../context/reference-sources/material-web/slider/internal/slider.ts) |
| Material Components for Android | [RangeSlider.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/RangeSlider.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/slider/RangeSlider.java) |
| Material Components for Android | [BaseSlider.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/BaseSlider.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/slider/BaseSlider.java) |
| Compose Material 3 | [Slider.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt) |
| Angular Material | [slider.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/slider/slider.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/slider/slider.ts) |
| Flutter material_ui | [range_slider.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/range_slider.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/range_slider.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: Slider.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Slider.md)
- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Android XML importer and runtime support (Wave 5A / Wave 9B / Wave 10A)

`tools/import-android-layout.py` lowers `com.google.android.material.slider.RangeSlider` to `Kind::SeekBar` with **end-to-end `RULE_RANGE_SLIDER = 250` emission** (`"<v0>,<v1>"`), mapping `android:valueFrom`/`valueTo`/`value` to `min`/`max`/`progress`, validating `android:stepSize` and Material 3 slider attributes, and resolving `app:values` (directly or via `@style`) from inline CSV/bracketed lists (`"20,80"`, `"[15.5, 65.5]"`), `@string/`, or `@array/` references backed by qualified/unqualified `<array>` and `<integer-array>` resources (supporting `@integer/` item refs, `f`/`F` suffixes, and AOSP qualifiers such as `values-sw600dp`, while strictly rejecting duplicate array definitions, non-`<item>` children, and malformed values). Values are clamped and ordered inside `[min, max]` (with 1-element lists setting `v1 = max`, and omission/`@null`/missing `@array/` defaulting to `[min + 0.25*(max - min), min + 0.75*(max - min)]`). Plain `Slider` and `SeekBar` omit rule `250`.

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, `RULE_RANGE_SLIDER = 250` (`"<v0>,<v1>"` on `Kind::SeekBar`) and `LayoutState.range_values: HashMap<String, [f32; 2]>` are independently implemented against pinned Apache-2.0 references — Material Components for Android [`RangeSlider.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/RangeSlider.java), [`BaseSlider.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/BaseSlider.java), and [`slider/res/values/tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/slider/res/values/tokens.xml) at commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed` (retained [LICENSE](../reference-licenses/material-android/LICENSE), local `context/reference-sources/material-android/lib/java/com/google/android/material/slider/` and `context/reference-repos/material-android/lib/java/com/google/android/material/slider/`) and Compose Material 3 [`Slider.kt`](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt) at commit `1448d64974931a04c9fba96ac630cbd6b96cda36` (retained [LICENSE.txt](../reference-licenses/compose-material3/LICENSE.txt)):

- **State & interaction:** Seeds `[v0, v1]` once into `LayoutState.range_values[key]` (clamped to `[min, max]` and ordered `v0 <= v1`). On pointer press, captures the nearest thumb (with RTL-aware directional disambiguation when `v0 == v1`), tracks it in `state.range_slider_active_thumb`, enforces non-crossing bounds (`min <= v0 <= v1 <= max`) throughout dragging, and returns `Some(self.id)` when either endpoint changes. When focused, keyboard `Left`/`Right`/`Up`/`Down`/`Home`/`End` (RTL-mirrored) adjusts the active thumb (defaulting to thumb 1).
- **Geometry, visual tokens & RTL:** Density-scaled (`LayoutState.effective_density()`) `4dp` rounded track (`8dp` track inset, `11dp` thumb travel inset), inactive `surface_variant` and active `primary` range segment spanning `[x0, x1]` (plus optional tick marks when a tick rule is present), two `20dp` (`10dp` radius) circular `primary` thumbs with Material level-1 shadow when enabled (`0.38` alpha when disabled), `40dp` circular hover/press/focus state-layer halos per thumb, custom `progressTint`/`progressBackgroundTint`/`thumbTint` overrides, and full RTL mirroring.
- **Motion:** Both thumbs follow pointer and keyboard input directly with no trailing delay ([Direct manipulation](../motion.md#motion-profiles)); per-thumb state layers use the shared state-layer effects and snap under reduced motion.
- **Validation evidence:**
  - Wave 14B native Rust test: `expressive_range_slider_enforces_min_separation_and_renders_two_handles_with_gaps_and_bubble` in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs) (verifies `Frame::expressive_range_slider` with `SliderExpressiveOptions` sanitizing initial `min_separation = 20.0`, rendering two `4 × 44dp` handles with `16dp` tracks and `6dp` gaps, enforcing `min_separation` on `Tab` + `End`/`Home` keyboard input, and displaying the `inverse_surface` value bubble on focus).
  - Wave 15A/9B Rust XML runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`288` passing tests; `872` default / `878` all-features Rust tests across `76` suites): `range_slider_rule_drags_both_thumbs_clamps_order_and_mirrors_rtl_and_density` and `wave15a_slider_options_step_min_sep_expressive_and_bubbles` (verifying `RULE_SLIDER_OPTIONS = 267` discrete `step_size`, `min_separation = 20` clamping during pointer drag, `label_behavior` dual value bubbles `"20"` / `"80"`, and Expressive `16dp` track + `4 × 44dp` handle geometry).
  - Python XML importer tests (`223` passing tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py)): `test_wave10a_range_slider_rule_250_and_array_resources` and `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269`.
  - Static visual captures (Mesa software GLES): Page-2 Light & Dark (`bhoo-m3-showcase-page2.png`, `bhoo-m3-showcase-page2-dark.png`) and Page-3 Light & Dark (`bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png` via [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) `--page2` / `--page3`; no Waydroid pixel-by-pixel comparison has been performed).
- **Remaining limitations:** More than two thumbs, `minSeparationUnit` (pixel-distance separation), custom value-bubble label formatters, independent Tab focus per thumb in Android XML (native `expressive_range_slider` supports per-thumb Tab focus via `[u64; 2]` IDs), and Waydroid visual comparison remain open. See [Android XML import](../android-xml.md) and [licenses and attribution](../licenses-and-attribution.md).
