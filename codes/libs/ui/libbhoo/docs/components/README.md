# Component Specification Index

Status: proposed design specifications. These documents do not implement widgets or an animation engine. The [implementation record](../implementation-progress.md) lists the first control/motion pass and its tested behavior. No family is claimed complete against its full specification.

Each catalog family has one Markdown file; its variants remain in that file. Shared foundations and desktop additions have separate files too. Names, priorities and motion profiles follow the [catalog](../components.md). All animation behavior references the [shared motion design](../motion.md).

Each specification includes a **Guidelines and visual references** section. Links point to component-specific Material documentation, relevant framework guides, or accessibility/desktop integration standards. Available upstream images and animated examples are linked directly, with local snapshot assets preferred where present. Related-pattern visuals are labeled, and entries without a verified image say so explicitly. These references illustrate upstream designs; Bhoo-specific visual designs remain separate work.

Each specification also has a **Downloaded reference implementations** section with selected source files, pinned upstream links, and local links into the ignored `context` folder. Direct family references and related desktop building blocks are distinguished. See the [source inventory](../reference-implementations.md) and [license review](../licenses-and-attribution.md) before adapting code; downloading a reference does not implement that component in Bhoo.

## Shared component contract

Treat the individual files as initial design contracts, not complete API designs. Before implementation, fill in exact layout/token values, state transitions, semantic roles and Rust API signatures appropriate to that family.

- Inputs: define pointer, applicable keyboard/touch/IME behavior and cancellation; decorative primitives do not become controls.
- Focus: preserve stable identity through reordering and return focus predictably when an overlay closes or focused content is removed.
- Accessibility: provide meaningful names, roles, values and state; announce functional changes without waiting for an animation. Avoid duplicate announcements from decorative children.
- Text and direction: support shaped labels, logical leading/trailing layout and mixed-direction text; input widgets additionally need grapheme-aware selection and IME.
- Ownership: application values stay with the caller; transient visual state is keyed by stable widget/property IDs and cleaned up on removal.
- Motion: use shared tokens/profiles, interruption and reduced-motion behavior; a still or offscreen component must not keep a frame loop alive.
- Validation: test functional correctness separately from visual transitions; test clipping, scale changes and resource lifetime where relevant.

## Specifications

### Actions

- [Button](button.md)
- [Icon button](icon-button.md)
- [Floating action button](floating-action-button.md)
- [Segmented selection / toggle group](segmented-selection-toggle-group.md)
- [Button group](button-group.md)
- [Split button](split-button.md)
- [Floating action menu](floating-action-menu.md)

### Selection and input

- [Checkbox](checkbox.md)
- [Radio group](radio-group.md)
- [Switch](switch.md)
- [Chip](chip.md)
- [Slider](slider.md)
- [Range slider](range-slider.md)
- [Text field](text-field.md)
- [Multiline text field](multiline-text-field.md)
- [Search](search.md)
- [Select](select.md)
- [Autocomplete](autocomplete.md)
- [Form-field container](form-field-container.md)
- [Date picker](date-picker.md)
- [Date-range picker](date-range-picker.md)
- [Time picker](time-picker.md)

### Navigation and toolbars

- [Top app bar](top-app-bar.md)
- [Bottom app bar](bottom-app-bar.md)
- [Navigation bar](navigation-bar.md)
- [Navigation rail](navigation-rail.md)
- [Navigation drawer](navigation-drawer.md)
- [Tabs](tabs.md)
- [Menu](menu.md)
- [Toolbar](toolbar.md)
- [Floating toolbar](floating-toolbar.md)

### Surfaces and containment

- [Card](card.md)
- [Dialog](dialog.md)
- [Bottom sheet](bottom-sheet.md)
- [Side sheet](side-sheet.md)
- [Divider](divider.md)
- [List and list item](list-and-list-item.md)
- [Carousel](carousel.md)
- [Expansion panel / accordion](expansion-panel-accordion.md)
- [Scaffold](scaffold.md)

### Communication and feedback

- [Badge](badge.md)
- [Progress indicator](progress-indicator.md)
- [Expressive loading indicator](expressive-loading-indicator.md)
- [Snackbar](snackbar.md)
- [Tooltip](tooltip.md)
- [Banner](banner.md)

### Data and application patterns

- [Data table](data-table.md)
- [Sort header](sort-header.md)
- [Paginator](paginator.md)
- [Tree](tree.md)
- [Stepper](stepper.md)
- [Grid list / image list](grid-list-image-list.md)
- [Avatar](avatar.md)
- [Scrollbar](scrollbar.md)
- [Refresh indicator](refresh-indicator.md)

### Visual and interaction foundations

- [Typography / text](typography-text.md)
- [Icon](icon.md)
- [Surface and elevation](surface-and-elevation.md)
- [State layer / ripple](state-layer-ripple.md)
- [Focus ring](focus-ring.md)
- [Theme tokens](theme-tokens.md)
- [Overlay and scrim](overlay-and-scrim.md)

### SpectreOS desktop additions

- [Virtualized list](virtualized-list.md)
- [Virtualized grid](virtualized-grid.md)
- [Scroll area / viewport](scroll-area-viewport.md)
- [Popover](popover.md)
- [Command palette](command-palette.md)
- [Application tile](application-tile.md)
- [Context-menu integration](context-menu-integration.md)
- [Shortcut hint](shortcut-hint.md)
- [Split pane / resizable divider](split-pane-resizable-divider.md)
- [Breadcrumb](breadcrumb.md)
- [Numeric field / spin box](numeric-field-spin-box.md)
- [Color picker](color-picker.md)
- [File chooser integration](file-chooser-integration.md)
- [Empty/error state](empty-error-state.md)
- [Notification surface](notification-surface.md)
