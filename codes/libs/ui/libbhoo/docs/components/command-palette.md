# Command palette

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | SpectreOS desktop additions |
| Kind | SpectreOS desktop proposal |
| Priority | P0 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Search, grouped commands, shortcuts, keyboard-first activation.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Track query, grouped commands, active command and shortcuts; keep async results and IME edits separate.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Use one overlay reveal and bounded active-row feedback; do not animate each search result.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Type rapidly while moving selection and confirm activation uses the current command ID.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Related Material Web guide](../material-design/upstream/docs/components/text-field.md)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/)

- [Local upstream visual example (related pattern)](../material-design/upstream/docs/components/images/textfield/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Related building blocks only; this is a SpectreOS-specific proposal, not an upstream component implementation.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [text-field.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/textfield/internal/text-field.ts) | [Downloaded file](../../../../context/reference-sources/material-web/textfield/internal/text-field.ts) |
| Material Components for Android | [SearchView.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/search/SearchView.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/search/SearchView.java) |
| Compose Material 3 | [SearchBar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt) |
| Angular Material | [autocomplete.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/autocomplete/autocomplete.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/autocomplete/autocomplete.ts) |
| Flutter material_ui | [search_anchor.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/search_anchor.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/search_anchor.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Bhoo catalog and reference implementations](../components.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Command palette preview](http://127.0.0.1:8766/#/command-palette).

Demonstrates: Editable application query, filtered commands and actual selection outcome.

Remaining scope: Global shortcut adapter, grouped results and full keyboard-first dialog model.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
