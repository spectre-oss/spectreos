# Icon

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Shared visual/interaction foundation |
| Priority | Shared foundation |
| Motion profile | [Static](../motion.md#motion-profiles) |

## Scope and variants

Sizing, color, accessible label supplied by the parent.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Define size, alignment, color and decorative/informative use; the action owner provides labels where appropriate.

Implemented API: `Frame::paint_material_icon(bounds, MaterialIcon::Palette, color)` paints a decorative official Google Material Symbol with a centered square design box. `Frame::material_icon_button(id, MaterialIcon::Add, "Add item", options)` adds button state, focus and a separate accessible action label. `MaterialIcon::ALL` and `name()` enumerate the 40 bundled symbols. The subset covers current gallery and app/messaging controls; it does not include the entire Google Fonts catalog. SVG originals, Apache-2.0 license and pinned upstream metadata are in `codes/libs/ui/libbhoo/assets/material-symbols`. Geometry is shared by the native and WASM renderers; no font/network request is needed.

## Motion and animation

Static by default; explicit icon morphs need compatible geometry or a simple crossfade.

Use the shared [static profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Remain static; the component owns no animation clock in either mode.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Swap a state icon and verify baseline, semantics and hit target remain stable.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/icon.md)
- [Material design guidance linked by upstream](https://m3.material.io/styles/icons/overview)

- [Local upstream visual example](../material-design/upstream/docs/components/images/icon/hero.gif)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Shared infrastructure reference; these files are not standalone widgets.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [icon.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/icon/internal/icon.ts) | [Downloaded file](../../../../context/reference-sources/material-web/icon/internal/icon.ts) |
| Compose Material 3 | [Icon.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Icon.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Icon.kt) |
| Angular Material | [icon.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/icon/icon.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/icon/icon.ts) |
| Flutter material_ui | [icon_button.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/icon_button.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/icon_button.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: icon.md](../material-design/upstream/docs/components/icon.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Icon preview](http://127.0.0.1:8766/#/icon).

Demonstrates: Static presentation symbols without an external icon font.

Remaining scope: Vector icon resource API and exact Material symbol geometry.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
