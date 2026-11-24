# Progress indicator

Status: Linear/circular determinate/indeterminate progress (Compose reference timelines), Wave 14B native M3 Expressive wavy linear & circular progress (`WavyProgressOptions`), and Wave 14C/15A Android XML `RULE_WAVY_PROGRESS = 263` implemented; buffered wavy tracks, complete token APIs, and Android device visual parity remain open.

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Continuous](../motion.md#motion-profiles) |

## Scope and variants

Linear/circular; determinate and indeterminate.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Distinguish determinate progress from unknown duration; expose progress values or busy state and cancellation where applicable.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Animate determinate changes without fabricating work; indeterminate motion runs only while visible and busy.

Use the shared [continuous profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Stop rotation/morphing and show a static busy indicator with accessible status; determinate values still update.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Hide or complete progress and verify the animation stops and the final semantic value is correct.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/progress.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/progress-indicators)
- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ProgressIndicator.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/progress-indicators/overview/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/progress/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [linear-progress.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/progress/internal/linear-progress.ts) | [Downloaded file](../../../../context/reference-sources/material-web/progress/internal/linear-progress.ts) |
| Material Web | [circular-progress.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/progress/internal/circular-progress.ts) | [Downloaded file](../../../../context/reference-sources/material-web/progress/internal/circular-progress.ts) |
| Material Components for Android | [BaseProgressIndicator.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/progressindicator/BaseProgressIndicator.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/progressindicator/BaseProgressIndicator.java) |
| Compose Material 3 | [ProgressIndicator.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ProgressIndicator.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ProgressIndicator.kt) |
| Angular Material | [progress-bar.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/progress-bar/progress-bar.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/progress-bar/progress-bar.ts) |
| Angular Material | [progress-spinner.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/progress-spinner/progress-spinner.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/progress-spinner/progress-spinner.ts) |
| Flutter material_ui | [progress_indicator.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/progress_indicator.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/progress_indicator.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: progress.md](../material-design/upstream/docs/components/progress.md)
- [Material Android: ProgressIndicator.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ProgressIndicator.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Native M3 Expressive wavy progress and Android XML support (Wave 14B / Wave 14C / Wave 15A)

- **Native M3 Expressive APIs (Wave 14B, [`progress.rs`](../../src/systems/material/progress.rs), [`widgets.rs`](../../src/systems/material/mod.rs), re-exported at crate root in [`lib.rs`](../../src/lib.rs)):**
  - `WavyProgressOptions` (`wavelength`, `amplitude`, `wave_speed`, `track_thickness`, `stop_indicator_size`, `track_gap`): pinned to MDC-Android `BaseProgressIndicatorSpec` defaults (`WavyProgressOptions::linear()`: `40dp` wavelength, `3dp` amplitude, `40dp/s` speed, `4dp` thickness/stop/gap; `WavyProgressOptions::circular()`: `16dp` wavelength, `3dp` amplitude, `40dp/s` speed, `4dp` thickness/stop/gap).
  - `wavy_amplitude_ramp(progress)` smoothly tapers wave amplitude to `0` near `0%` (`< 0.1`) and `100%` (`> 0.9`) via smoothstep, flattening active geometry cleanly at the endpoints.
  - `Frame::wavy_linear_progress(id, label, value, options)` renders a traveling sinusoidal `DrawCommand::Line` active polyline (`primary`), a `track_gap`-separated straight inactive track (`secondary_container`), and a trailing `stop_indicator_size` circular dot (`primary`, omitted at `100%`), supporting both determinate (`Some(v)`) and indeterminate (`None`) modes.
  - `Frame::wavy_circular_progress(id, label, value, options)` (`48dp` default container) renders a polar sinusoidal `DrawCommand::Line` wave separated by `track_gap` from a straight `DrawCommand::Arc` inactive track (`secondary_container`), flattening to a smooth full `Arc` at `100%`.
  - **Reduced motion:** Retains a static wave without traveling phase and stops requesting continuous redraws (`!ui.needs_redraw()`).
- **Android XML `LinearWavyProgressIndicator` / `CircularWavyProgressIndicator` (`RULE_WAVY_PROGRESS = 263`) (Wave 14C / Wave 15A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - Emits `"<wavelength_dp>|<amplitude_dp>|<wave_speed_dp>|<stop_size_dp>|<track_gap_dp>"` on `Kind::ProgressBar`. At runtime, renders sinusoidal linear or circular waves (or flat active bars with explicit gap + stop dot when `wavelength == 0`), respecting `progressTint`/`progressBackgroundTint` and reduced motion.
- **Validation:**
  - Wave 14B Rust tests in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs): `wavy_progress_options_match_pinned_m3_defaults_and_amplitude_ramp`, `wavy_linear_progress_draws_sinusoid_gap_stop_dot_and_respects_ramps_and_reduced_motion`, `wavy_circular_progress_draws_sinusoidal_polar_wave_inactive_arc_and_respects_reduced_motion`, plus `crate_root_reexports_wave14b_and_wave15b_types` in [`tests/wave15b_composition.rs`](../../tests/wave15b_composition.rs).
  - Wave 15A/14C tests: `wave15a_loading_indicator_and_wavy_progress` in [`tests/android_xml.rs`](../../tests/android_xml.rs) and `test_wave14c_loading_indicator_and_wavy_progress_rules_262_263` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
  - Static Light/Dark Mesa GLES Page-3 captures: `context/android-xml-comparison/bhoo-m3-showcase-page3.png` and `bhoo-m3-showcase-page3-dark.png` (via [`android_m3_showcase.rs`](../../../../codes/libs/ui/libbhoo/examples/android_m3_showcase.rs) `--page3`).
- **Remaining limitations:** Secondary buffered wavy tracks, custom wave-amplitude transition springs on indeterminate-to-determinate switching, RTL linear wave direction validation, and real-device visual parity remain open. See [Android XML import](../android-xml.md).
