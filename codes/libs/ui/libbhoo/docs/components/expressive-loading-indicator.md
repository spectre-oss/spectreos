# Expressive loading indicator

Status: Rust loading widget implemented with matched upstream geometry and reference timelines; renderer validation and production qualification are recorded separately.

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Continuous](../motion.md#motion-profiles) |

## Scope and variants

Shape-changing loading feedback distinct from progress.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Represent busy state independently of the decorative shape sequence; define when loading starts and completes.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Morph shapes on a shared visible-only timeline; reduced motion shows a static busy treatment.

Use the shared [continuous profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Stop rotation/morphing and show a static busy indicator with accessible status; determinate values still update.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Remove the loading view mid-cycle and verify its animation state is reclaimed and no redraw continues.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/LoadingIndicator.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/loading-indicator/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/loadingindicator/loadingindicator-configurations.gif)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [LoadingIndicator.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/loadingindicator/LoadingIndicator.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/loadingindicator/LoadingIndicator.java) |
| Compose Material 3 | [LoadingIndicator.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/LoadingIndicator.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/LoadingIndicator.kt) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: LoadingIndicator.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/LoadingIndicator.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Expressive loading indicator preview](http://127.0.0.1:8766/#/expressive-loading-indicator).

Demonstrates: Seven matched upstream polygon morphs, spring/rotation timing, determinate circle-to-soft-burst and contained/uncontained variants.

Remaining scope: Assistive-technology announcements and hardware performance qualification.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## M3 contained token alignment and Android XML support (Wave 14B / Wave 14C / Wave 15A)

- **Native contained M3 tokens (Wave 14B, [`loading.rs`](../../src/loading.rs)):** `Frame::loading_indicator` with `contained = true` now draws the `48 × 48dp` circular container (`24dp` radius) using `theme.primary_container` and the morphing `DrawCommand::Loading` shape using `theme.on_primary_container` (uncontained continues to draw `theme.primary` with no container fill).
- **Android XML `LoadingIndicator` (`RULE_LOADING_INDICATOR = 262`) (Wave 14C / Wave 15A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):** `tools/import-android-layout.py` lowers `com.google.android.material.loadingindicator.LoadingIndicator` to `Kind::ProgressBar` (`indeterminate: true`) with `RULE_LOADING_INDICATOR = 262` (`"<contained:0|1>"`). At runtime, `"1"` renders the `primaryContainer` circular container + `onPrimaryContainer` morphing shape and `"0"` renders the uncontained `primary` morphing shape (honoring `progressTint` and reduced motion).
- **Validation:**
  - Wave 14B Rust test: `loading_indicator_contained_uses_m3_primary_container_and_on_primary_container_tokens` in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs).
  - Wave 15A/14C tests: `wave15a_loading_indicator_and_wavy_progress` in [`tests/android_xml.rs`](../../tests/android_xml.rs) and `test_wave14c_loading_indicator_and_wavy_progress_rules_262_263` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
  - Static Light/Dark Mesa GLES Page-3 captures: `context/android-xml-comparison/bhoo-m3-showcase-page3.png` and `bhoo-m3-showcase-page3-dark.png`.
- **Remaining limitations:** Custom `indicatorSize`/`containerWidth`/`containerHeight` overrides beyond bounding-box sizing, `showDelay`/`minHideDelay` timers, custom polygon sequences, and hardware GPU performance qualification remain open. See [Android XML import](../android-xml.md).
