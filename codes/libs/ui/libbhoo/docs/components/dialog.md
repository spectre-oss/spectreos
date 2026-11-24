# Dialog

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Basic alert/confirmation and custom/full-screen content.

`OverlayState::fullscreen` makes a dialog fill its host viewport with square
corners. The dialog example's “Full-screen dialog” switch demonstrates a header
with Close, title, Save, a divider and content. It still uses the shared modal
focus and dismissal behavior. Android's reference describes this anatomy but
does not provide a dedicated full-screen Material implementation. The example
uses a simple fade; exact platform transition, system insets, scrolling content,
complete responsive behavior remain open. In the example, Close opens a nested
discard confirmation; Keep editing returns to the full-screen dialog.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Own modal focus, accessible title/description, dismissal policy and result; nested dialogs need explicit ownership.

`OverlayState::dismiss_policy` selects Escape and outside-press dismissal
independently. The default permits both; `ExplicitOnly` requires an action or
caller-controlled close. The dialog example exposes a “Require explicit choice”
switch. Modal input remains contained while implicit dismissal is disabled.
The Rust tests cover each policy and an explicit Save, and the browser test
checks the explicit-choice switch with Escape, outside press and Cancel.

For a separately rendered child modal, set the parent overlay's `suspended`
field while the child is open. The parent retains its state and focus return
point but ignores input and implicit dismissal; the child owns focus until it
closes. The caller is responsible for ordering and clearing suspension. Tests
cover Escape and focus return, and the browser example tests a full-screen
parent with a discard confirmation. Arbitrary automatic overlay stacking and
deeply nested ownership are not yet implemented.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Fade the scrim and gently reveal the surface; exiting content becomes noninteractive immediately.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

Modal dialogs can supply `OverlayState::accessible_title` and
`accessible_description`. The overlay emits non-focusable dialog and description
semantics while open, removes them on close, and keeps background targets disabled.
The playground adapter exposes the name, description and modal state on the
canvas container and mirrors enabled dialog buttons as accessible DOM controls;
Tab wraps within those controls, Escape dismisses, and activating a control is
dispatched to Rust. Focus returns to the canvas on close. The single-canvas
gallery uses a separate DOM dialog bridge; its worker sends semantic snapshots
to the main thread and receives focus and activation events for the owning Rust
context. The same bridge runs directly on the main-thread fallback.
`cargo test --manifest-path
codes/libs/ui/libbhoo/Cargo.toml --test dialog_semantics` and
`node tools/test-bhoo-dialog-semantics.mjs` and
`node tools/test-bhoo-canvas-dialog-a11y.mjs` check these transitions and keyboard
activation, including worker and main-thread canvas paths. Other control roles,
the native bridge and automatic nested modal ownership remain incomplete, and actual
screen-reader use has not been verified; these checks do not establish full modal
accessibility.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Close during entry and confirm the final frame, focus restoration and pending result occur once.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/dialog.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/dialogs)
- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Dialog.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/dialogs/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/dialog/hero.webp)
- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/dialogs/dialogs-types-hero.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [dialog.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/dialog/internal/dialog.ts) | [Downloaded file](../../../../context/reference-sources/material-web/dialog/internal/dialog.ts) |
| Material Components for Android | [MaterialAlertDialogBuilder.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/dialog/MaterialAlertDialogBuilder.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/dialog/MaterialAlertDialogBuilder.java) |
| Compose Material 3 | [AlertDialog.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AlertDialog.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AlertDialog.kt) |
| Angular Material | [dialog.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/dialog/dialog.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/dialog/dialog.ts) |
| Flutter material_ui | [dialog.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/dialog.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/dialog.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: dialog.md](../material-design/upstream/docs/components/dialog.md)
- [Material Android: Dialog.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Dialog.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Dialog preview](http://127.0.0.1:8766/#/dialog).

Demonstrates: Modal surface/scrim with Angular comparison timing (150/75ms), dismissal, focus containment and restoration; the Material Web profile remains available.

Remaining scope: Additional accessible dialog child roles, native bridge,
screen-reader verification and automatic nested ownership.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Native M3 tokens and Android XML `AlertDialog` support (Wave 14C / Wave 15A / Wave 15B)

- **Native M3 `OverlayKind::Dialog` tokens (Wave 15B, [`composition.rs`](../../src/systems/material/mod.rs)):** Standard modal dialogs render with M3 `28dp` corner radius and `surface_high` (`surfaceContainerHigh`) container surface; fullscreen dialogs (`OverlayState::fullscreen = true`) retain `0dp` square corners over the viewport with `surface_high`. Verified by `dialog_uses_m3_28dp_radius_and_surface_container_high` in [`tests/wave15b_composition.rs`](../../tests/wave15b_composition.rs).
- **Android XML `AlertDialogLayout` / `MaterialAlertDialog*` (`RULE_ALERT_DIALOG = 264`) (Wave 14C / Wave 15A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):**
  - `tools/import-android-layout.py` emits `RULE_ALERT_DIALOG = 264` (`"<icon_name_or_empty>"`) on `AlertDialogLayout` or `MaterialAlertDialog*`-styled containers. At runtime, `Kind::Linear` paints `28dp`-rounded `surfaceContainerHigh` chrome with elevation shadow, `24dp` default padding, and an optional centered `24dp` top `MaterialIcon` offsetting title/body children downward.
  - **Validation:** Rust runtime test `wave15a_alert_dialog_and_snackbar_chrome` in [`tests/android_xml.rs`](../../tests/android_xml.rs), Python importer test `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), and static Light/Dark Mesa GLES Page-3 captures (`context/android-xml-comparison/bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png`).
  - **Remaining limitations:** Automatic scrolling dialog body dividers, stacked button-bar overflow wrapping, custom non-`MaterialIcon` dialog icon drawables, automatic nested modal ownership, and device visual parity remain open. See [Android XML import](../android-xml.md).
