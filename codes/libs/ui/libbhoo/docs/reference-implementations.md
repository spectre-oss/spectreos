# Downloaded Reference Implementations

Checked 2026-11-08. Five source-only snapshots are available under `context/reference-sources`. No new Git clones were created for this download. Existing reference checkouts from earlier work remain separate.

These are selected library source files at exact commits, not complete repository mirrors, SDKs or buildable standalone distributions. Binary/media assets, dependencies, tests, localization/generated icon payloads and most build configuration were omitted. Some supporting files are included to make component state and motion logic understandable.

The ignored [local manifest](../../../context/reference-sources/manifest.json) records a SHA-256 digest, byte count and immutable raw URL for every downloaded file. The tracked [reference index](reference-sources.json) records source revisions, reviewed license paths and per-component matches.

## Source snapshots

| Toolkit | Revision | Files | Source bytes | Reviewed project license |
| --- | --- | --- | --- | --- |
| Material Web | [d872455c4be5](https://github.com/material-components/material-web/tree/d872455c4be56022da2095349b30ef46e0b55bc3) | 713 | 1,988,953 | [Apache-2.0](reference-licenses/material-web/LICENSE) |
| Material Components for Android | [66b0eeb53afe](https://github.com/material-components/material-components-android/tree/66b0eeb53afeb6025ffbbd37734b08e5bce197ed) | 454 | 5,157,897 | [Apache-2.0](reference-licenses/material-android/LICENSE) |
| Compose Material 3 | [1448d6497493](https://github.com/androidx/androidx/tree/1448d64974931a04c9fba96ac630cbd6b96cda36) | 255 | 4,707,900 | [Apache-2.0](reference-licenses/compose-material3/LICENSE.txt) |
| Angular Material | [249b3ee5c8b3](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c) | 577 | 2,433,089 | [MIT](reference-licenses/angular-material/LICENSE) |
| Flutter material_ui | [565c65738619](https://github.com/flutter/packages/tree/565c6573861992fbd225ca138bbc2d3b8aeb5d00) | 201 | 5,482,633 | [BSD-3-Clause](reference-licenses/flutter-material/AUTHORS) |

Project-level licenses do not replace file-specific notices. Read the [license and attribution review](licenses-and-attribution.md) for source reuse requirements and scope limits. Flutter's snapshot uses the current standalone `material_ui` package source rather than assuming Material still lives only in the Flutter SDK.

## Per-component source index

Each component's own document includes pinned source and local download links. Desktop entries link related building blocks where available; they do not claim an equivalent complete Material implementation. An unmatched entry remains explicitly unverified.

| Bhoo specification | Reference snapshots with selected files |
| --- | --- |
| [Application tile](components/application-tile.md) | Material Web, Compose Material 3, Flutter material_ui (related building blocks) |
| [Autocomplete](components/autocomplete.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Avatar](components/avatar.md) | Flutter material_ui |
| [Badge](components/badge.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Banner](components/banner.md) | Flutter material_ui |
| [Bottom app bar](components/bottom-app-bar.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Bottom sheet](components/bottom-sheet.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Breadcrumb](components/breadcrumb.md) | No exact source selected; guidelines only |
| [Button group](components/button-group.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Button](components/button.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Card](components/card.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Carousel](components/carousel.md) | Material Components for Android, Compose Material 3, Flutter material_ui |
| [Checkbox](components/checkbox.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Chip](components/chip.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Color picker](components/color-picker.md) | Material Components for Android, Compose Material 3, Flutter material_ui (related building blocks) |
| [Command palette](components/command-palette.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui (related building blocks) |
| [Context-menu integration](components/context-menu-integration.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui (related building blocks) |
| [Data table](components/data-table.md) | Angular Material, Flutter material_ui |
| [Date picker](components/date-picker.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Date-range picker](components/date-range-picker.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Dialog](components/dialog.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Divider](components/divider.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Empty/error state](components/empty-error-state.md) | Compose Material 3, Angular Material, Flutter material_ui (related building blocks) |
| [Expansion panel / accordion](components/expansion-panel-accordion.md) | Material Components for Android, Angular Material, Flutter material_ui |
| [Expressive loading indicator](components/expressive-loading-indicator.md) | Material Components for Android, Compose Material 3 |
| [File chooser integration](components/file-chooser-integration.md) | No exact source selected; guidelines only |
| [Floating action button](components/floating-action-button.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Floating action menu](components/floating-action-menu.md) | Compose Material 3, Angular Material, Flutter material_ui |
| [Floating toolbar](components/floating-toolbar.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Focus ring](components/focus-ring.md) | Material Web, Compose Material 3, Flutter material_ui |
| [Form-field container](components/form-field-container.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Grid list / image list](components/grid-list-image-list.md) | Angular Material, Flutter material_ui |
| [Icon button](components/icon-button.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui |
| [Icon](components/icon.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui |
| [List and list item](components/list-and-list-item.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui |
| [Menu](components/menu.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui |
| [Multiline text field](components/multiline-text-field.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Navigation bar](components/navigation-bar.md) | Material Components for Android, Compose Material 3, Flutter material_ui |
| [Navigation drawer](components/navigation-drawer.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Navigation rail](components/navigation-rail.md) | Material Components for Android, Compose Material 3, Flutter material_ui |
| [Notification surface](components/notification-surface.md) | Compose Material 3, Flutter material_ui (related building blocks) |
| [Numeric field / spin box](components/numeric-field-spin-box.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui (related building blocks) |
| [Overlay and scrim](components/overlay-and-scrim.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Paginator](components/paginator.md) | Angular Material, Flutter material_ui |
| [Popover](components/popover.md) | Material Web, Compose Material 3, Angular Material, Flutter material_ui (related building blocks) |
| [Progress indicator](components/progress-indicator.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Radio group](components/radio-group.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Range slider](components/range-slider.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Refresh indicator](components/refresh-indicator.md) | Compose Material 3, Flutter material_ui |
| [Scaffold](components/scaffold.md) | Compose Material 3, Flutter material_ui |
| [Scroll area / viewport](components/scroll-area-viewport.md) | Compose Material 3, Flutter material_ui (related building blocks) |
| [Scrollbar](components/scrollbar.md) | Compose Material 3, Flutter material_ui |
| [Search](components/search.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Segmented selection / toggle group](components/segmented-selection-toggle-group.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Select](components/select.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Shortcut hint](components/shortcut-hint.md) | No exact source selected; guidelines only |
| [Side sheet](components/side-sheet.md) | Material Components for Android, Angular Material |
| [Slider](components/slider.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Snackbar](components/snackbar.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Sort header](components/sort-header.md) | Angular Material, Flutter material_ui |
| [Split button](components/split-button.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Split pane / resizable divider](components/split-pane-resizable-divider.md) | Material Components for Android, Compose Material 3, Flutter material_ui (related building blocks) |
| [State layer / ripple](components/state-layer-ripple.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Stepper](components/stepper.md) | Angular Material, Flutter material_ui |
| [Surface and elevation](components/surface-and-elevation.md) | Material Web, Material Components for Android, Compose Material 3, Flutter material_ui |
| [Switch](components/switch.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Tabs](components/tabs.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Text field](components/text-field.md) | Material Web, Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Theme tokens](components/theme-tokens.md) | Material Web, Material Components for Android, Compose Material 3, Flutter material_ui |
| [Time picker](components/time-picker.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Toolbar](components/toolbar.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Tooltip](components/tooltip.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Top app bar](components/top-app-bar.md) | Material Components for Android, Compose Material 3, Angular Material, Flutter material_ui |
| [Tree](components/tree.md) | Angular Material |
| [Typography / text](components/typography-text.md) | Material Components for Android, Compose Material 3, Flutter material_ui |
| [Virtualized grid](components/virtualized-grid.md) | No exact source selected; guidelines only |
| [Virtualized list](components/virtualized-list.md) | Flutter material_ui (related building blocks) |

## Motion implementation references

The component links lead to real source, including state handling where present. For shared motion, start with these downloaded modules:

- [Compose Material 3 — MotionScheme.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MotionScheme.kt)
- [Flutter material_ui — motion.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/motion.dart)
- [Flutter material_ui — curves.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/curves.dart)
- [Material Components for Android — MotionUtils.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/motion/MotionUtils.java)
- [Material Web — ripple.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/ripple/ripple.ts)

Bhoo's [motion specification](motion.md) remains a proposal. Downloading or studying an upstream solver does not implement it in Bhoo. Selection behavior, motion tokens and framework scheduling are different layers; review all relevant layers before porting one.

## Refresh procedure

Choose the upstream revision explicitly, download the selected paths and every applicable license/NOTICE file without rewriting their contents, regenerate the file manifest and this index, then check hashes and all local component links. Update the recorded source revisions together with the license review. Do not silently replace a pinned reference or claim a source-only selection is a full build tree.
