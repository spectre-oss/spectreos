# Expansion panel / accordion

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Layout](../motion.md#motion-profiles) |

## Scope and variants

Disclosure header and expandable content.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Expose expanded state on each header; define single/multiple open policy and focus behavior for collapsing content.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Interpolate content height and indicator rotation; collapsed content stops receiving input at logical collapse.

Use the shared [layout profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Apply final geometry immediately and retain the same stable focus/selection identity; no sliding or size tween.

### Angular comparison implementation

The browser comparison allows multiple open panels. Expanded headers grow from
48 to 64 px, and their 128 px content area and 16 px surrounding gaps animate
over 225 ms with `cubic-bezier(0.4, 0, 0.2, 1)`. Adjacent margins collapse to
the larger gap, including while animating. The expanded header suppresses its
hover background. Interrupted transitions use CSS reversal shortening; reduced
motion snaps to the final geometry. These choices follow [Angular expansion styles](https://github.com/angular/components/blob/main/src/material/expansion/expansion-panel.scss)
and [header styles](https://github.com/angular/components/blob/main/src/material/expansion/expansion-panel-header.scss).

`node tools/test-bhoo-expansion-motion.mjs` samples opening, closing, adjacent
panels and an interrupted reversal against Angular; `node tools/test-bhoo-expansion-layout.mjs`
compares the expanded middle panel and hover surface. Closed panels use square
inner shadow edges and rounded outside edges. `node tools/test-bhoo-expansion-edges.mjs`
checks 240 edge pixels and header positions across all eight open/closed states
in light and dark themes. When all three are open, Angular adds a browser
scrollbar that changes its available width; that width difference is excluded
from the geometry check and remains open. This is a browser fixture, not
certification of all native rendering or accessibility behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Collapse a panel containing focus and restore it to the header before hiding descendants.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/)
- [Angular expansion panel guide](https://material.angular.dev/components/expansion/overview)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [ExpandableWidgetHelper.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/expandable/ExpandableWidgetHelper.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/expandable/ExpandableWidgetHelper.java) |
| Angular Material | [expansion-panel.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/expansion/expansion-panel.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/expansion/expansion-panel.ts) |
| Flutter material_ui | [expansion_panel.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/expansion_panel.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/expansion_panel.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Angular Material: expansion/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/expansion/)
- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Expansion panel / accordion preview](http://127.0.0.1:8766/#/expansion-panel-accordion).

Demonstrates: Independent disclosures with rotating chevrons and reversible clipped height transitions.

Remaining scope: Dedicated disclosure semantics, full content layout and focus-on-collapse policy.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

The Angular comparison uses an immediate 8% hover state layer on collapsed headers. The middle header has square corners; only the outer top and bottom corners of the group are rounded. Keyboard-visible focus uses a 12% state layer, and expanded headers suppress this background. Browser pixel checks cover each collapsed row in light and dark themes.
