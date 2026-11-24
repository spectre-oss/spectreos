# Surface and elevation

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Shared visual/interaction foundation |
| Priority | Shared foundation |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Theme surface roles, shape, borders, depth.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Represent semantic surface roles, shape, border and depth without owning application interaction.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Interpolate surface color or elevation only on meaningful state changes; geometry remains stable.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Change theme or elevation and verify contrast and ordering without perpetual redraws.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/elevation.md)
- [Material design guidance linked by upstream](https://m3.material.io/styles/elevation)

- [Local upstream visual example](../material-design/upstream/docs/components/images/elevation/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Shared infrastructure reference; these files are not standalone widgets.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [elevation.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/elevation/internal/elevation.ts) | [Downloaded file](../../../../context/reference-sources/material-web/elevation/internal/elevation.ts) |
| Material Components for Android | [ElevationOverlayProvider.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/elevation/ElevationOverlayProvider.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/elevation/ElevationOverlayProvider.java) |
| Compose Material 3 | [Surface.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Surface.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Surface.kt) |
| Flutter material_ui | [material.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/material.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/material.dart) |
| Flutter material_ui | [elevation_overlay.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/elevation_overlay.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/elevation_overlay.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: elevation.md](../material-design/upstream/docs/components/elevation.md)
- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Surface and elevation preview](http://127.0.0.1:8766/#/surface-and-elevation).

Demonstrates: Surface role/tint levels and approximate static shadows.

Remaining scope: Physical shadow rendering and complete elevation tokens.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML shape, gradient, 9-patch & dynamic elevation support (Wave 17A / Wave 17B)

In [`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../../src/systems/android/xml/mod.rs), [`tools/android_png.py`](../../../../tools/android_png.py), and [`tools/import-android-layout.py`](../../../../tools/import-android-layout.py):

- **True radial & sweep `<shape>` gradients (`RULE_GRADIENT = 207`):** Emits `"radial|<cx>,<cy>,<radius_dp>|<start>|<center>|<end>"` and `"sweep|<cx>,<cy>|<start>|<center>|<end>"`; runtime rasterizes radial, sweep, and cut-corner linear gradients into bounded cached `Image`s (`LayoutState::gradient_images`, capped at `256×256` / `32` entries / `4 MB`) with `SRC_IN` `background_tint` and rounded, oval SDF, or chamfered cut-corner SDF clipping.
- **Chamfered (`cut`) `ShapeAppearance` corners (`RULE_CORNER_FAMILY = 278`):** Resolves `cornerFamily*` across `shapeAppearance`, `shapeAppearanceOverlay`, direct attributes, and styles (`"<tl>,<tr>,<br>,<bl>"`, `"r"`/`"c"`); runtime paints anti-aliased cut/mixed fills via bounded cached `Image`s (`LayoutState::cut_shape_images`, capped at `256×256` / `48` entries / `4 MB`) and miter-inset outlines via `DrawCommand::Line`/`Arc` across general views, `Button`/`MaterialButton`/`ExtendedFloatingActionButton`/`Chip`, and `MaterialCardView`.
- **Multi-segment `.9.png` stretch bands (`251..=254`):** `decode_nine_patch_bands` extracts all contiguous `#FF000000` stretch runs per axis; runtime slices alternating density-scaled fixed segments and stretchable segments distributing remaining pixels proportionally across all stretch bands.
- **`AppBarLayout` dynamic `liftOnScroll` (`RULE_APP_BAR_LIFT_ON_SCROLL = 279`):** Animates (`150ms` ease-out, instant snap under reduced motion) from `0dp` flat to M3 level-2 (`3dp`) shadow + tonal `surface_high` / `8%` primary tint lift when sibling or target scroll offset exceeds `0.5`.
- **Validation:** Verified by `wave17a_radial_and_sweep_gradients_raster_cache_and_clipping`, `wave17a_cut_corners_shape_appearance_fill_outline_and_widgets`, `wave17a_multi_segment_nine_patch_stretch_bands`, and `wave17a_app_bar_layout_lift_on_scroll` in [`tests/android_xml.rs`](../../tests/android_xml.rs) plus `test_wave17b_radial_and_sweep_gradients`, `test_wave17b_cut_corner_family_rule_278`, `test_wave17b_multi_band_nine_patch`, and `test_wave17b_app_bar_lift_on_scroll_rule_279` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py). Hardware GPU performance profiling and physical device shadow/color parity remain open. See [Android XML support](../android-xml.md).
