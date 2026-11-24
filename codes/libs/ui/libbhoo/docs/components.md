# Bhoo Component Catalog and Implementation References

Status: research and proposed inventory, checked 2026-11-08. This document adds design scope, not implemented Rust widgets.

The local Material Web snapshot documents **20 families**, including visual primitives such as elevation, focus rings, and ripples. It is not the full Material ecosystem. The inventory below combines official Material implementations, consolidates related APIs into families, and separately proposes desktop extensions for SpectreOS. It is a broad baseline, not a claim to enumerate every third-party implementation on the internet.

The [implementation record](implementation-progress.md) lists the initial stateful controls, motion, input and rendering work. Individual files mark partial implementations explicitly. None is yet claimed to satisfy its entire cross-toolkit specification; every family now has an experimental browser scene; full family implementations remain incomplete. See the [demo coverage matrix](demo-coverage.md).

## Reference implementations

| Key | Primary reference | Why it is useful |
| --- | --- | --- |
| Web | [Material Web component docs](material-design/upstream/docs/components/) | Existing local reference for controls, styles, and tokens |
| Android | [Material Components for Android](https://github.com/material-components/material-components-android/tree/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components) | Broader component docs, including newer expressive families |
| Compose | [Material components in Compose](https://developer.android.com/develop/ui/compose/components) | Behavior, state models, adaptive UI, and declarative API examples |
| Angular | [Angular Material source](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material) | Application controls, tables, trees, form fields, and interaction patterns |
| Flutter | [Material widget catalog](https://docs.flutter.dev/ui/widgets/material) and [API reference](https://api.flutter.dev/flutter/material/) | Composable variants and application-level patterns |

The pinned Android and Angular directory inventories are recorded in [research-sources.json](material-design/research-sources.json). Online framework docs are live references. The local Web snapshot retains its previously recorded commit. Platform-specific APIs and experimental components require individual review before adoption; presence in a source tree is not a stability guarantee.

The [official Material guidelines](https://m3.material.io/) remain the design reference. Framework catalogs also contain older Material patterns and framework extensions. Their presence below does not mean they are all current Material 3 specification components.

## Priorities

- **P0:** launcher foundation; implement after text, focus, and input infrastructure are ready.
- **P1:** common SpectreOS applications and settings.
- **P2:** advanced, adaptive, expressive, or less urgent patterns.

Priorities are Bhoo planning proposals, not upstream recommendations. Variants in a row belong to one family; they are not counted as separate implemented components.

## Actions

| Family | Variants or design scope | Reference | Priority |
| --- | --- | --- | --- |
| [Button](components/button.md) | Text, outlined, filled, tonal, elevated; optional leading icon | Web `button.md`; Android `CommonButton.md` | P0 |
| [Icon button](components/icon-button.md) | Standard, filled, tonal, outlined; optional selected state | Web `icon-button.md`; Android `IconButton.md` | P0 |
| [Floating action button](components/floating-action-button.md) | Small, regular, large; extended label form | Android `FloatingActionButton.md`, `ExtendedFloatingActionButton.md` | P2 |
| [Segmented selection / toggle group](components/segmented-selection-toggle-group.md) | Single and multiple selection | Compose segmented buttons; Android `ToggleButtonGroup.md`; Angular `button-toggle/` | P1 |
| [Button group](components/button-group.md) | Grouped related actions; expressive spacing and shape behavior | Android `ButtonGroup.md` | P2 |
| [Split button](components/split-button.md) | Primary action plus a separate secondary-action menu trigger | Android `SplitButton.md` | P1 |
| [Floating action menu](components/floating-action-menu.md) | Expandable group of floating actions | Android `FloatingActionButtonMenu.md` | P2 |

## Selection and input

[Rating](components/rating.md) adds a whole-star selection widget inspired by MUI; the current implementation is partial.

| Family | Variants or design scope | Reference | Priority |
| --- | --- | --- | --- |
| [Checkbox](components/checkbox.md) | Checked, unchecked, indeterminate | Web `checkbox.md` | P1 |
| [Radio group](components/radio-group.md) | Mutually exclusive choices; group keyboard model | Web `radio.md`; Angular `radio/` | P1 |
| [Switch](components/switch.md) | On/off; label and optional icon | Web `switch.md` | P1 |
| [Chip](components/chip.md) | Assist, filter, input, suggestion; selected/removable where applicable | Web `chip.md`; Flutter chip APIs | P1 |
| [Slider](components/slider.md) | Continuous/discrete; ticks, value indicator | Web `slider.md`; Android `Slider.md` | P1 |
| [Range slider](components/range-slider.md) | Two values and ordered thumbs | Android `Slider.md`; Flutter `RangeSlider` | P1 |
| [Text field](components/text-field.md) | Filled/outlined; labels, helper/error text, prefix/suffix, icons | Web `text-field.md` | P0 |
| [Multiline text field](components/multiline-text-field.md) | Wrapping, selection, scrolling, variable height | Web text-field textarea example; Flutter `TextField` | P1 |
| [Search](components/search.md) | Search entry, suggestions, results surface, clear action | Android `Search.md`; Compose search | P0 |
| [Select](components/select.md) | Filled/outlined; keyboard selection and popup list | Web `select.md`; Angular `select/` | P1 |
| [Autocomplete](components/autocomplete.md) | Editable entry with filtered suggestions | Angular `autocomplete/`; Flutter `Autocomplete` | P0 |
| [Form-field container](components/form-field-container.md) | Label, description, validation, error state, optional counter | Angular `form-field/`; Flutter `TextFormField` | P1 |
| [Date picker](components/date-picker.md) | Calendar and text-entry modes | Android `DatePicker.md`; Compose date picker | P1 |
| [Date-range picker](components/date-range-picker.md) | Start/end dates and range validation | Compose date picker; Flutter `DateRangePickerDialog` | P2 |
| [Time picker](components/time-picker.md) | Clock and text-entry modes; locale-aware formatting | Android `TimePicker.md`; Angular `timepicker/` | P1 |

## Navigation and toolbars

[Page transition](components/page-transition.md) adds an initial Beer CSS-inspired page entry.

[Link](components/link.md) adds text link feedback and caller-owned navigation.

| Family | Variants or design scope | Reference | Priority |
| --- | --- | --- | --- |
| [Top app bar](components/top-app-bar.md) | Title, navigation affordance, actions; compact/expanded forms | Android `TopAppBar.md`; Compose app bars | P1 |
| [Bottom app bar](components/bottom-app-bar.md) | Bottom actions and optional FAB relationship | Android `BottomAppBar.md` | P2 |
| [Navigation bar](components/navigation-bar.md) | Primary destination items and selected state | Android `BottomNavigation.md`; Compose navigation bar | P2 |
| [Navigation rail](components/navigation-rail.md) | Leading-edge navigation for wider layouts | Android `NavigationRail.md`; Compose navigation rail | P1 |
| [Navigation drawer](components/navigation-drawer.md) | Persistent or modal destination list | Android `NavigationDrawer.md`; Compose navigation drawer | P1 |
| [Tabs](components/tabs.md) | Primary/secondary styling, selected indicator, overflow/scrolling | Web `tabs.md` | P1 |
| [Menu](components/menu.md) | Items, separators, selected/checkable items, submenus where supported | Web `menu.md`; Angular `menu/` | P0 |
| [Toolbar](components/toolbar.md) | Inline groups of application actions | Angular `toolbar/`; Android `DockedToolbar.md` | P1 |
| [Floating toolbar](components/floating-toolbar.md) | Floating action surface; placement and collapse behavior | Android `FloatingToolbar.md`, `DockedFloatingToolbars.md` | P2 |

## Surfaces and containment

[Box layout](components/layout-box.md) provides a padded surface. [Stack layout](components/layout-stack.md) and [Container layout](components/layout-container.md) add initial placement primitives. [Grid layout](components/layout-grid.md) adds an initial equal-column layout primitive.

| Family | Variants or design scope | Reference | Priority |
| --- | --- | --- | --- |
| [Card](components/card.md) | Filled, outlined, elevated; optional interaction | Android `Card.md`; Compose cards | P1 |
| [Dialog](components/dialog.md) | Basic alert/confirmation and custom/full-screen content | Web `dialog.md`; Android `Dialog.md` | P1 |
| [Bottom sheet](components/bottom-sheet.md) | Standard and modal; dismissal and focus containment | Android `BottomSheet.md`; Compose bottom sheets | P2 |
| [Side sheet](components/side-sheet.md) | Standard and modal supplementary content | Android `SideSheet.md` | P1 |
| [Divider](components/divider.md) | Horizontal/vertical, full-width or inset | Web `divider.md`; Flutter dividers | P0 |
| [List and list item](components/list-and-list-item.md) | Leading/trailing content, supporting text, selection | Web `list.md`; Android `List.md` | P0 |
| [Carousel](components/carousel.md) | Scrolling collection with item sizing and alignment behavior | Android `Carousel.md`; Compose carousel | P2 |
| [Expansion panel / accordion](components/expansion-panel-accordion.md) | Disclosure header and expandable content | Angular `expansion/`; Flutter `ExpansionPanel` | P1 |
| [Scaffold](components/scaffold.md) | Compose content regions with bars, drawers, sheets, and feedback | Compose scaffold; Flutter `Scaffold` | P1 |

## Communication and feedback

[Alert](components/alert.md) provides inline severity feedback. [Skeleton](components/skeleton.md) provides initial loading placeholders inspired by MUI.

| Family | Variants or design scope | Reference | Priority |
| --- | --- | --- | --- |
| [Badge](components/badge.md) | Dot or count/label attached to another element | Android `BadgeDrawable.md`; Compose badges | P1 |
| [Progress indicator](components/progress-indicator.md) | Linear/circular; determinate and indeterminate | Web `progress.md`; Android `ProgressIndicator.md` | P1 |
| [Expressive loading indicator](components/expressive-loading-indicator.md) | Shape-changing loading feedback distinct from progress | Android `LoadingIndicator.md` | P2 |
| [Snackbar](components/snackbar.md) | Transient message with optional action and timeout | Android `Snackbar.md`; Angular `snack-bar/` | P1 |
| [Tooltip](components/tooltip.md) | Plain/rich content, hover/focus activation, dismissal | Android `Tooltip.md`; Compose tooltips | P0 |
| [Banner](components/banner.md) | Persistent contextual notice and actions; legacy/framework pattern | Android `Banner.md`; Flutter `MaterialBanner` | P2 |

## Data and application patterns

[Image media](components/media.md) adds an initial bounded RGBA renderer; video remains open.

[Avatar group](components/avatar-group.md) adds overlapping initial avatars. [Pagination](components/pagination.md) adds numbered page navigation inspired by MUI.

These entries broaden Bhoo beyond Material Web. Several are implementation-library or older Material patterns rather than dedicated current M3 specification families.

| Family | Variants or design scope | Reference | Priority |
| --- | --- | --- | --- |
| [Data table](components/data-table.md) | Headers, rows, selection, sorting hooks | Angular `table/`; Android `DataTable.md`; Flutter `DataTable` | P1 |
| [Sort header](components/sort-header.md) | Direction state, keyboard activation, sort announcement | Angular `sort/` | P1 |
| [Paginator](components/paginator.md) | Page size, range label, previous/next controls | Angular `paginator/` | P1 |
| [Tree](components/tree.md) | Nested/flat models, expansion, selection | Angular `tree/` | P1 |
| [Stepper](components/stepper.md) | Linear/nonlinear workflow with validation | Angular `stepper/`; Flutter `Stepper` | P2 |
| [Grid list / image list](components/grid-list-image-list.md) | Structured tiles, captions, image placement | Angular `grid-list/`; Android `ImageList.md` | P1 |
| [Avatar](components/avatar.md) | Image, initials, or fallback icon | Flutter `CircleAvatar` | P1 |
| [Scrollbar](components/scrollbar.md) | Visible position, thumb interaction, hover behavior | Flutter `Scrollbar` | P0 |
| [Refresh indicator](components/refresh-indicator.md) | Pull-to-refresh feedback; touch-oriented interaction | Flutter `RefreshIndicator` | P2 |

## Visual and interaction foundations

[Hover wave](components/hover-wave.md) adds initial Beer CSS-inspired hover feedback.

[Shape rotation](components/shape-rotation.md) adds linear animated shape rotation. [Expressive shapes](components/expressive-shapes.md) adds 37 static Beer CSS-derived silhouettes.

[Grow transition](components/transition-grow.md) adds a nonuniform scale and opacity reveal. [Slide transition](components/transition-slide.md) adds a viewport-relative reveal. [Collapse transition](components/transition-collapse.md) adds a vertical clipped reveal. [Zoom transition](components/transition-zoom.md) adds centered scaling. [Fade transition](components/transition-fade.md) provides initial timed opacity transitions.

These are shared services or primitives, not standalone application widgets to count as additional controls.

| Foundation | Scope | Reference |
| --- | --- | --- |
| [Typography / text](components/typography-text.md) | Semantic styles, measurement, selection-ready text layout | Web theming typography; Android `MaterialTextView.md` |
| [Icon](components/icon.md) | Sizing, color, accessible label supplied by the parent | Web `icon.md` |
| [Surface and elevation](components/surface-and-elevation.md) | Theme surface roles, shape, borders, depth | Web `elevation.md`; Flutter `Material` |
| [State layer / ripple](components/state-layer-ripple.md) | Hover, focus, press and selected feedback | Web `ripple.md`; Angular core/ripple |
| [Focus ring](components/focus-ring.md) | Visible keyboard focus independent of hover | Web `focus-ring.md` |
| [Theme tokens](components/theme-tokens.md) | Color, type, shape, spacing, density, motion | Web theming; Compose Material 3 theme |
| [Overlay and scrim](components/overlay-and-scrim.md) | Positioning, z-order, dismissal, focus restoration | Compose dialogs/sheets; Angular CDK overlay |

## SpectreOS desktop additions

The following are Bhoo proposals driven by the launcher and desktop environment. They are **not presented as official Material components**.

| Proposed family | Why SpectreOS needs it | Priority |
| --- | --- | --- |
| [Virtualized list](components/virtualized-list.md) | Large application/result collections without building hidden rows | P0 |
| [Virtualized grid](components/virtualized-grid.md) | Application icon grid and other large tile collections | P1 |
| [Scroll area / viewport](components/scroll-area-viewport.md) | Clipping, scrolling, nested wheel behavior, visibility reporting | P0 |
| [Popover](components/popover.md) | Anchored custom content beyond a menu or tooltip | P0 |
| [Command palette](components/command-palette.md) | Search, grouped commands, shortcuts, keyboard-first activation | P0 |
| [Application tile](components/application-tile.md) | App icon, name, status, selection, activation affordance | P0 |
| [Context-menu integration](components/context-menu-integration.md) | Desktop secondary-click and keyboard menu invocation | P0 |
| [Shortcut hint](components/shortcut-hint.md) | Consistent keycap rendering and shortcut descriptions | P0 |
| [Split pane / resizable divider](components/split-pane-resizable-divider.md) | Adjustable sidebar/content and inspector layouts | P1 |
| [Breadcrumb](components/breadcrumb.md) | Hierarchical paths and navigation | P1 |
| [Numeric field / spin box](components/numeric-field-spin-box.md) | Validated numeric editing and stepping | P1 |
| [Color picker](components/color-picker.md) | Color selection for appearance and creative applications | P2 |
| [File chooser integration](components/file-chooser-integration.md) | Open/save flows, using desktop services where appropriate | P1 |
| [Empty/error state](components/empty-error-state.md) | Consistent loading failure, no results, and recovery actions | P0 |
| [Notification surface](components/notification-surface.md) | Desktop notification content and actions; system integration separate | P2 |

## Per-component specifications

Every family and foundation above has its own linked Markdown specification. See the [specification index](components/README.md) and [shared motion design](motion.md). The runtime effects and standard spatial engine are implemented; full component choreography remains incomplete. The [coverage matrix](demo-coverage.md) separates demonstrated behavior from target specifications.

## Required specification for each family

Before implementation, document anatomy, variants, state transitions, public API, layout and clipping behavior, focus/keyboard rules, pointer/touch behavior where applicable, semantic accessibility roles, RTL/text behavior, theme tokens, and motion/reduced-motion behavior.

Include resource and performance expectations appropriate to the component. Lists and grids require virtualization tests; overlays require dismissal and focus-restoration tests; text inputs require selection and IME tests. A painted visual preview alone does not make a component complete.

## Suggested delivery order

1. Shared text shaping, focus, semantics, scrolling, and overlay foundations.
2. Launcher controls: buttons/icons, search/autocomplete, virtualized results, app tiles, menus, popovers, tooltips, and shortcut hints.
3. Settings/application controls: selection inputs, form fields, tabs, navigation, cards, dialogs, and feedback.
4. Data-heavy UI: tables, trees, pagination, sorting, and resizable panes.
5. Advanced pickers, adaptive surfaces, and expressive variants when real application requirements justify them.

This order deliberately follows SpectreOS needs rather than copying a framework's entire exported API. Research references inform the design; adopting upstream code requires reviewing the license of that particular source.

| [Android horizontal scroll](components/android-horizontal-scroll.md) | XML horizontal scrolling container | Android `HorizontalScrollView` | P1 |
| [Android XML checkbox](components/android-checkbox.md) | Checked, unchecked and disabled | Android `CheckBox` | P1 |
| [Android XML button](components/android-button.md) | Text button and disabled state | Android `Button` | P1 |
| [Android XML switch](components/android-switch.md) | On, off and disabled | Android `Switch` | P1 |
| [Android XML seek bar](components/android-seekbar.md) | Integer range and disabled state | Android `SeekBar` | P1 |
| [Android XML progress bar](components/android-progressbar.md) | Determinate and indeterminate | Android `ProgressBar` | P1 |
| [Android XML radio group](components/android-radio-group.md) | Single selection and disabled state | Android `RadioGroup` | P1 |
| [Android XML rating bar](components/android-ratingbar.md) | Fractional stars and read-only indicator | Android `RatingBar` | P1 |
| [Android XML toggle button](components/android-togglebutton.md) | On, off and disabled | Android `ToggleButton` | P1 |
| [Android XML FrameLayout](components/android-frame-layout.md) | Stacking and layout gravity | Android `FrameLayout` | P1 |
| [Android XML LinearLayout](components/android-linear-layout.md) | Horizontal and vertical child layout | Android `LinearLayout` | P1 |
| [Android XML ImageView](components/android-imageview.md) | Fit center and center crop | Android `ImageView` | P1 |
| [Android XML ImageButton](components/android-imagebutton.md) | Image-backed button | Android `ImageButton` | P1 |
| [Android XML TextView](components/android-textview.md) | Wrapped text | Android `TextView` | P1 |

| [Android XML Space](components/android-space.md) | Fixed and weighted empty space | Android `Space` | P1 |
| [Android XML CheckedTextView](components/android-checked-textview.md) | Host-controlled checked text | Android `CheckedTextView` | P1 |
| [Android XML ScrollView](components/android-scrollview.md) | Vertical scrolling container | Android `ScrollView` | P1 |
| [Android XML EditText](components/android-edittext.md) | Editable text field | Android `EditText` | P1 |
| [Android XML ViewSwitcher](components/android-viewswitcher.md) | Switch between child views | Android `ViewSwitcher` | P1 |
| [Android XML ViewFlipper](components/android-viewflipper.md) | Automatically cycle child views | Android `ViewFlipper` | P1 |
| [Android XML TextSwitcher](components/android-textswitcher.md) | Animate text changes | Android `TextSwitcher` | P1 |
| [Android XML ImageSwitcher](components/android-imageswitcher.md) | Animate image changes | Android `ImageSwitcher` | P1 |
| [Android XML Chronometer](components/android-chronometer.md) | Count up or down | Android `Chronometer` | P1 |
| [Android XML AbsoluteLayout](components/android-absolute-layout.md) | Fixed-position layout | Android `AbsoluteLayout` | P2 |
| [Android XML ZoomButton](components/android-zoombutton.md) | Repeating zoom action | Android `ZoomButton` | P2 |
| [Android XML DigitalClock](components/android-digitalclock.md) | Display local time | Android `DigitalClock` | P2 |
| [Android XML TextClock](components/android-textclock.md) | Display wall-clock time | Android `TextClock` | P1 |
| [Android XML AnalogClock](components/android-analogclock.md) | Analog wall-clock display | Android `AnalogClock` | P2 |
| [Android XML ZoomControls](components/android-zoomcontrols.md) | Zoom in/out with long-press repeat | Android `ZoomControls` | P2 |
| [Android XML RelativeLayout](components/android-relative-layout.md) | Rule-based child placement | Android `RelativeLayout` | P1 |

| [Android XML AutoCompleteTextView](components/android-autocomplete.md) | Editable text with suggestions | Android `AutoCompleteTextView` | P1 |
| [Android XML GridView](components/android-gridview.md) | Adapter-backed scrolling grid | Android `GridView` | P1 |
| [Android XML ListView](components/android-listview.md) | Adapter-backed scrolling list | Android `ListView` | P1 |
| [Android XML Spinner](components/android-spinner.md) | Adapter-backed dropdown selection | Android `Spinner` | P1 |

- [Android XML MultiAutoCompleteTextView](components/android-multi-autocomplete.md) — initial comma-tokenized suggestions.

| [Android XML QuickContactBadge](components/android-quick-contact-badge.md) | Contact image and host activation | Android `QuickContactBadge` | P1 |

| [Android XML MediaController](components/android-media-controller.md) | Host-backed playback controls | Android `MediaController` | P1 |

| [Android XML VideoView](components/android-video-view.md) | Host video frame surface and controller | Android `VideoView` | P1 |
