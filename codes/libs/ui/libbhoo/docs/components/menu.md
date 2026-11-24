# Menu

Status: Plain action-menu demo and `Frame::menu_item` implemented and compared
against Angular Material 22.2.6. Check/radio items, submenus and full menu scope
remain incomplete.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Material/framework-inspired component family |
| Priority | P0 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Items, separators, selected/checkable items, submenus where supported.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Support named items, separators, check/radio items and submenus; arrows, Enter and Escape follow a consistent menu model.

`Frame::menu_item(id, label, enabled)` provides a 48 px transparent action row,
14 px medium text with 0.096 px tracking and 12 px side padding. Rows share
the menu container surface; they do not paint ordinary list-item backgrounds.
Hover/focus state layers apply immediately, matching the pinned reference.
Disabled items retain a disabled menu-item semantic node and do not activate.
The demo supports arrow/Home/End navigation, Enter/Space activation, Escape,
Tab dismissal, outside press and focus restoration. Background trigger input
and feedback are suppressed while the menu owns the pointer backdrop.

`OverlayState::initial_focus` selects the first enabled focus target;
`transform_origin` can specify the anchor pivot. Menu and menu-item roles are
preserved in the renderer-independent semantic snapshot. This is not yet a
complete native accessibility or ARIA adapter.

## Motion and animation

The Angular comparison profile uses the actual pinned menu keyframes:
120 ms cubic-bezier(0, 0, 0.2, 1) from scale 0.8 / opacity 0 to scale 1 /
opacity 1. Exit retains full geometry and fades over 100 ms linear after a
25 ms delay. The default pivot is the top-left anchor. Reduced motion snaps.
Pointer interaction is blocked during entry while enabled appearance and
keyboard targets are preserved. Closing removes input immediately; exiting
visuals cannot activate actions. A nonmodal menu does not animate a scrim.

Closing during entry reproduces the pinned Angular `animationcancel` teardown:
the panel detaches immediately. Reopening restarts the enter keyframe rather
than using spring reversal. The original Material Web overlay profile remains
available through `OverlayStyle::Material3`.

Run `node tools/test-bhoo-menu-motion.mjs` with Chromium CDP on port 9223.
Fifteen phase-aligned Angular/WASM samples check bounds, opacity, text geometry
and the delayed exit; additional checks cover interrupted entry and reduced
motion. Evidence is saved in ignored `context/bhoo-menu-motion.json`. Rust tests
exercise entry pointer blocking, enabled semantics, disabled items, keyboard
focus origin, action commits, Escape restoration and return to idle.

Remaining work includes nested/checkable menu behavior, type-ahead, locale/RTL,
scrolling and collision/flip placement for arbitrary anchors and long menus.
CSS group-opacity compositing, font rasterization and native shadows can still
differ; these tests do not certify complete pixel parity.

Source: [pinned Angular menu module](https://unpkg.com/@angular/material@22.2.6/fesm2022/menu.mjs),
[menu overview](https://material.angular.dev/components/menu/overview), and
[upstream styles](https://github.com/angular/components/blob/main/src/material/menu/menu.scss).

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Traverse a submenu by keyboard and dismiss during entry without leaving focus in a hidden menu.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/menu.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/menus)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/menu/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [Typeahead controller](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/menu/internal/controllers/typeaheadController.ts) | [Downloaded file](../../../../context/reference-sources/material-web/menu/internal/controllers/typeaheadController.ts) |
| Material Web | [menu.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/menu/internal/menu.ts) | [Downloaded file](../../../../context/reference-sources/material-web/menu/internal/menu.ts) |
| Compose Material 3 | [Menu.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Menu.kt) |
| Angular Material | [menu.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/menu/menu.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/menu/menu.ts) |
| Angular Material | [menu-trigger.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/menu/menu-trigger.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/menu/menu-trigger.ts) |
| Flutter material_ui | [menu_anchor.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/menu_anchor.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/menu_anchor.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: menu.md](../material-design/upstream/docs/components/menu.md)
- [Angular Material: menu/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/menu/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Menu preview](http://127.0.0.1:8766/#/menu).

Demonstrates: Anchored menu with reference 500/150ms growth/exit, staggered items, keyboard movement, 200 ms label typeahead, and immediate exit input rejection. Typeahead uses Unicode lowercase prefix matching, skips disabled entries, cycles from the focused item, and resets when focus moves externally. Spaces extend an active query; standalone Space activates the focused item. Complex text and IME composition are not supported.

Remaining scope: Submenus, complete menu role adapter, and text composition. The typeahead behavior follows the Apache-2.0 Material Web controller; licensing is recorded in the source inventory.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
