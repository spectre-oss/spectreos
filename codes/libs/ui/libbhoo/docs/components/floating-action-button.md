# Floating action button

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Layout](../motion.md#motion-profiles) |

## Scope and variants

Small (40 dp), regular (56 dp), medium (80 dp), and large (96 dp); extended label form. The medium and large icon sizes follow the pinned Compose reference (28 dp and 36 dp). Secondary FABs use explicit secondary-container and on-secondary-container theme roles, which may differ from the secondary roles. Default corner radii are 12, 16, 20 and 28 dp respectively; callers can override the corner radius. Medium and large extended variants use the pinned Compose padding, icon spacing and text sizes.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose one primary action; the extended variant adds a visible label; keep action identity through size changes.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

The icon FAB animates container width, icon position and label opacity when extending or collapsing, retaining the same action ID.

The width uses the fast spatial motion profile and the label uses the fast effects profile, matching the pinned Compose reference. Narrow containers keep the icon within the available bounds. Reversals start from the current presented state. Exact platform timing and interpolation remain to be compared.

**Reduced motion:** Apply final geometry immediately and retain the same stable focus/selection identity; no sliding or size tween.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Toggle extended form at narrow widths without clipping the label or duplicating focus.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/FloatingActionButton.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/floating-action-button)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/fabs/fab-expressive-light.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [fab.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/fab/internal/fab.ts) | [Downloaded file](../../../../context/reference-sources/material-web/fab/internal/fab.ts) |
| Material Components for Android | [FloatingActionButton.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/floatingactionbutton/FloatingActionButton.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/floatingactionbutton/FloatingActionButton.java) |
| Material Components for Android | [ExtendedFloatingActionButton.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/floatingactionbutton/ExtendedFloatingActionButton.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/floatingactionbutton/ExtendedFloatingActionButton.java) |
| Compose Material 3 | [FloatingActionButton.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButton.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButton.kt) |
| Angular Material | [button.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/button/button.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/button/button.ts) |
| Flutter material_ui | [floating_action_button.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/floating_action_button.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/floating_action_button.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: FloatingActionButton.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/FloatingActionButton.md)
- [Material Android: ExtendedFloatingActionButton.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/ExtendedFloatingActionButton.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Floating action button preview](http://127.0.0.1:8766/#/floating-action-button).

Demonstrates: Small, regular, large and extended creation actions, using the bundled Material add icon at 24 dp (32 dp for large) and a content-sized extended button. The focus ring, activation count, right-to-left extended layout, and hover elevation are shown in the demo. The examples use the primary, secondary, tertiary, and surface container color roles, including matching icon and state-layer colors. Shared Angular palettes include tertiary roles. The demo also offers lowered (1 dp resting, 3 dp hover) and bottom-app-bar (flat) elevation. The 6 dp resting and 8 dp hover targets follow the Apache-2.0 Compose FAB tokens; the three shadow layers interpolate between the local Apache-2.0 Angular elevation values. Press and keyboard focus return to 6 dp, and reduced motion snaps. The original glyph-based API remains available.

Remaining scope: Full icon and shape tokens, bidirectional text shaping, complete dynamic color resolution, exact platform shadow rendering and motion parity. The current elevation interpolation uses the shared effects spring rather than Compose’s exact animation.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML importer and runtime support (Wave 8A / Wave 8B / Wave 16A / Wave 17A / Wave 17B / Wave 22A / Wave 22B)

`tools/import-android-layout.py` lowers `com.google.android.material.floatingactionbutton.ExtendedFloatingActionButton` to `Kind::Button` emitting `RULE_EXTENDED_FAB = 242` (`anchor: "1"` extended by default; `"0"` when `app:extended="false"`, `@bool/` resolves to `false`, inherited via `@style`, or `app:extendMotionSpec="false"` is set without an explicit `app:extended`) and, in Wave 22B, `RULE_EXTENDED_FAB_EXT = 299` (`"<extended:0|1>|<collapsed_size_dp>|<strategy>"`, resolving `app:extended` `true`/`false`, `app:collapsedSize` `@dimen/` or dp defaulting to `56`, and `app:extendStrategy` `auto`/`wrap_content`/`match_parent`; omitted on defaults while preserving `RULE_EXTENDED_FAB = 242`). The importer sets a `16.0dp` default corner radius (overridden by `app:cornerRadius`, `app:shapeAppearance` / `app:shapeAppearanceOverlay` including full-pill `radius: -1.0`, asymmetric `RULE_CORNER_RADII = 206`, and Wave 17B chamfered `cut` `RULE_CORNER_FAMILY = 278`, or shape backgrounds) and M3 `primaryContainer` background (`#EADDFF`) when `android:background` is omitted, and validates `app:extended`, `app:extendMotionSpec`, `app:shrinkMotionSpec`, `app:showMotionSpec`, `app:hideMotionSpec`, `app:collapsedSize`, `app:extendStrategy` (`auto`, `wrap_content`, `match_parent`), `app:elevation` (forwarded to `RULE_ELEVATION = 193`), and `app:ensureMinTouchTargetSize` (`true`/`false`), rejecting invalid values strictly. (Icon-only `com.google.android.material.floatingactionbutton.FloatingActionButton` continues to lower to `Kind::Icon` with circular `radius: -1.0` and automatic M3 level-3 shadow.)

At runtime in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`, independently implemented against pinned Material Components for Android [`ExtendedFloatingActionButton.java`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/floatingactionbutton/ExtendedFloatingActionButton.java) and [`efab_tokens.xml`](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/floatingactionbutton/res/values/efab_tokens.xml) (commit `66b0eeb53afeb6025ffbbd37734b08e5bce197ed`, Apache-2.0; retained [LICENSE](../reference-licenses/material-android/LICENSE), local `context/reference-repos/material-android/lib/java/com/google/android/material/floatingactionbutton/`):

- **Geometry and visual tokens:** `56dp` wrap/minimum height and `80dp` extended minimum width (density-scaled via `LayoutState.effective_density()`), `16dp` default corner radius (respecting explicit `radius`, full-pill `-1.0`, asymmetric `RULE_CORNER_RADII = 206`, and Wave 17A chamfered `cut` corners `RULE_CORNER_FAMILY = 278` via cached `cut_shape_images` fills, `DrawCommand::Line`/`Arc` outlines, and cut/radial/sweep `RULE_GRADIENT = 207` backgrounds), level-3 elevation shadow when enabled (omitted when disabled, overridden by `RULE_ELEVATION = 193`), M3 `primaryContainer` (`#EADDFF` in `Material3Light`, `#4F378B` in `Material3Dark`) container fill and `onPrimaryContainer` (`#21005D` Light, `#EADDFF` Dark) foreground/icon/state-layer defaults (`0.38` alpha when disabled, overridden by explicit background, `RULE_BACKGROUND_TINT`, gradient, layer-list, or `textColor`), RTL-aware default padding `[16dp, 0, 20dp, 0]` in LTR (`[20dp, 0, 16dp, 0]` in RTL), and leading `24dp` icon (from `image_src` / bundled `MaterialIcon` / tinted `LayoutState.images` with `12dp` gap to label, or compound drawables, mirrored to physical right in RTL).
- **Animated shrink/extend, `collapsedSize`, `extendStrategy` & runtime override (`RULE_EXTENDED_FAB_EXT = 299` + `LayoutState::fab_extended`, Wave 22A/22B):** Callers can toggle extended/collapsed state at runtime via `LayoutState::fab_extended: HashMap<String, bool>` (overriding the static XML state from `242`/`299`). Collapsed geometry uses `collapsed_size_dp × collapsed_size_dp` (default `56 × 56dp`) with `[0, 0, 0, 0]` default padding, centered `24dp` icon, and hidden label text; extended width honors `extendStrategy` (`match_parent` expands to parent width while `auto`/`wrap_content` measure intrinsic pill width). Bounded `LayoutState.extended_fab_motion` (`64`-entry cap) smoothly animates width/height over `0.22s` ease-out between collapsed and extended dimensions — synchronized across intrinsic measurement and painting — while fading label opacity and honoring RTL alignment, and snapping immediately without intermediate frames under `MotionPreference::Reduced`.
- **`CoordinatorLayout` Snackbar-aware FAB dodge (Wave 16A) & `BottomAppBar` cradle/embed docking (Wave 22A):** When a visible bottom-docked `SnackbarContentLayout` (`RULE_SNACKBAR = 265`) is present in a `CoordinatorLayout` (`Kind::Frame`), bottom-aligned `ExtendedFloatingActionButton` and elevated bottom-aligned `FloatingActionButton` siblings shift upward above the snackbar (`snackbar_height + vertical_margins + 8dp` gap) so they never overlap the snackbar. When anchored to a `BottomAppBar` carrying `RULE_BOTTOM_APP_BAR_EXT = 300`, `CoordinatorLayout` docks the FAB into the bar's `cradle`/`embed` coordinate and slides it in sync when `hideOnScroll` is active (see [Bottom app bar](bottom-app-bar.md)).
- **Interaction and motion:** Clicking returns the XML ID (`Option<&'static str>`). Runtime `fab_extended` transitions animate width, height, and label opacity over `0.22s` ease-out and snap immediately under `MotionPreference::Reduced` (aligning with [motion.md](../motion.md#motion-profiles)); Snackbar dodge placement applies immediately.
- **Validation evidence:**
  - Functional Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`324` passing tests within `919` default / `925` all-features Rust tests across `76` suites): `extended_fab_rule_renders_m3_container_elevation_padding_overrides_and_collapsed`, `wave16a_coordinator_snackbar_aware_fab_dodge`, `wave17a_cut_corners_shape_appearance_fill_outline_and_widgets`, `wave22a_extended_fab_ext_width_animation_strategies_state_override_rtl_and_reduced_motion` (verifying `RULE_EXTENDED_FAB_EXT = 299`, `LayoutState::fab_extended` toggle, `0.22s` ease-out synchronized width/height animation & label fade, `collapsedSize`, `match_parent`/`wrap_content`/`auto` strategies, RTL alignment, and reduced-motion instant snap), and `wave22a_bottom_app_bar_ext_cradle_embed_cutout_icons_anchor_rtl_and_hide_on_scroll`.
  - Importer unit tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`252` passing tests, `0` diffs across `167` existing layouts): `test_wave8a_extended_fab_rule_242_and_corner_radius`, `test_wave17b_cut_corner_family_rule_278`, and Wave 22B `test_wave22b_extended_fab_ext_rule_299`.
  - Static visual captures (Mesa software GLES): Material 3 Page-1/Page-2 showcases (`bhoo-m3-showcase*.png`), pure-XML Gmail inbox (`gmail-xml-inbox.png`), and pure-XML Contacts home/empty (`contacts-xml-home.png`, `contacts-xml-empty.png`); no automated pixel-diff against Android devices has been performed.
- **Remaining Android XML limitations:** Custom XML `extendMotionSpec`/`shrinkMotionSpec`/`showMotionSpec`/`hideMotionSpec` animator resource evaluation, scroll-triggered auto-shrink, M3 Expressive medium/large extended FAB XML variants, and Waydroid visual comparison remain open. See [Android XML import](../android-xml.md) and [licenses and attribution](../licenses-and-attribution.md).
