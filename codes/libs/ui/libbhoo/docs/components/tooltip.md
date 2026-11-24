# Tooltip

Status: Partial implementation — M3 plain tooltip tokens (`inverse_surface`/`inverse_on_surface`/`4dp`), interactive `Frame::rich_tooltip` (Wave 14B), and Android XML `RULE_TOOLTIP_TEXT = 258` (Wave 14A); touch long-press reveal, rich-tooltip XML tags, and device parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Plain/rich content, hover/focus activation, dismissal.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Provide hover/focus triggers, show/hide delays and plain/rich semantics; interactive content needs its own focus policy.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Use a short overlay fade; waiting for the reveal delay must not request frames continuously.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

### Angular comparison implementation

For the single-line comparison fixture, the Angular profile centers a 24 px
high tooltip beneath its trigger with an 8 px gap, inverse-surface colors,
12 px text and 4 px corners. Pointer and keyboard focus both wait 500 ms.
The tooltip scales and fades from the trigger-facing edge over 150 ms, then
scales and fades out over 75 ms; reduced motion snaps. Escape suppresses it
until the pointer or focus leaves. The 8 px bridge allows the pointer to enter
the shown tooltip. See [Angular tooltip styles](https://github.com/angular/components/blob/main/src/material/tooltip/tooltip.scss)
and [tooltip behavior](https://github.com/angular/components/blob/main/src/material/tooltip/tooltip.ts).

`node tools/test-bhoo-tooltip.mjs` checks geometry, colors, delay, the hover
bridge and Escape; `node tools/test-bhoo-tooltip-motion.mjs` compares 13 timed
samples. Multi-line wrapping, every viewport/RTL placement, touch gestures and
native accessibility still need separate validation.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Move between triggers before the delay expires and verify stale tooltip timers do not reveal the wrong text.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Tooltip.md)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [TooltipDrawable.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/tooltip/TooltipDrawable.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/tooltip/TooltipDrawable.java) |
| Compose Material 3 | [Tooltip.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tooltip.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tooltip.kt) |
| Angular Material | [tooltip.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/tooltip/tooltip.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/tooltip/tooltip.ts) |
| Flutter material_ui | [tooltip.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/tooltip.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/tooltip.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: Tooltip.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Tooltip.md)
- [Compose component catalog](https://developer.android.com/develop/ui/compose/components)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Native M3 Plain/Rich Tooltip and Android XML support (Wave 14A / Wave 14B)

- **Native M3 Plain Tooltip token alignment (Wave 14B, [`widgets.rs`](../../src/systems/material/mod.rs)):** `Frame::tooltip` now uses M3 `theme.inverse_surface` background, `theme.inverse_on_surface` text, and `4dp` corner radius.
- **Native M3 `Frame::rich_tooltip` (Wave 14B, [`widgets.rs`](../../src/systems/material/mod.rs)):** `Frame::rich_tooltip(id, anchor, subhead, body, action_label) -> bool` schedules the `0.5s` hover reveal wakeup (`next_wakeup`), renders a `12dp`-rounded `surface_low` surface with a 3-layer level-2 elevation shadow, `14sp` bold (`weight >= 700`) subhead, `14sp` regular body, and an optional interactive `primary`-colored action button (preserving hover while the pointer moves onto the tooltip surface, returning `true` on action click, exposing `Role::Tooltip`, and dismissing on `Escape`).
- **Android XML `android:tooltipText` (`RULE_TOOLTIP_TEXT = 258`) (Wave 13A / Wave 14A, [`android_xml.rs`](../../src/systems/android/xml/mod.rs)):** Attaches plain tooltip reveal (`0.5s` hover delay via `next_wakeup`, immediate reveal on keyboard `Tab` focus including non-clickable views like `TextView`, `Role::Tooltip` semantics, and `Escape` suppression) to any XML view carrying `RULE_TOOLTIP_TEXT = 258`.
- **Validation:**
  - Wave 14B Rust tests in [`tests/wave14b_m3_expressive.rs`](../../tests/wave14b_m3_expressive.rs): `plain_tooltip_uses_m3_inverse_surface_tokens_and_4dp_radius` and `rich_tooltip_renders_m3_spec_and_handles_interactive_action_click_and_escape`.
  - Wave 14A/13A tests: `wave14a_tooltip_text_reveals_on_hover_delay_and_keyboard_focus` in [`tests/android_xml.rs`](../../tests/android_xml.rs) and `test_wave13a_transforms_text_style_ext_and_tooltip_rules_256_257_258` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py).
- **Remaining limitations:** Touch long-press activation, multi-action rich tooltip layouts, automatic caret arrow rendering, and Android device visual parity remain open. See [Android XML import](../android-xml.md).
