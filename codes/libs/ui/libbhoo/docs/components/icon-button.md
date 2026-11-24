# Icon button

Status: Partial implementation — named icon action primitive; complete glyph/resources and variant validation remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Actions |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Standard, filled, tonal, outlined; optional selected state.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Require an accessible action name; distinguish momentary action from an optional selected toggle; support Enter/Space.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Blend hover and press colors; selected icon changes may crossfade without moving the glyph baseline.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Verify icon-only buttons expose names and a toggle announces its selected state.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/icon-button.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/icon-buttons)
- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/IconButton.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/icon-buttons/overview)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/iconbutton/hero.webp)
- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/buttons/iconbuttons-expressive.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [icon-button.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/iconbutton/internal/icon-button.ts) | [Downloaded file](../../../../context/reference-sources/material-web/iconbutton/internal/icon-button.ts) |
| Compose Material 3 | [IconButton.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButton.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/IconButton.kt) |
| Angular Material | [icon-button.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/button/icon-button.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/button/icon-button.ts) |
| Flutter material_ui | [icon_button.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/icon_button.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/icon_button.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: icon-button.md](../material-design/upstream/docs/components/icon-button.md)
- [Material Android: IconButton.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/IconButton.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)
