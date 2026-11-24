# Browser demo coverage

All catalog entries have a live Rust scene. Eighteen use the initial widget APIs; the rest are experimental application compositions built from Bhoo primitives. A live preview is not evidence of complete family, accessibility or motion parity.

## Motion baseline

Effects use AndroidX StandardMotionTokens damping 1.0 / stiffness 1600. Geometry disclosures and overlay travel use standard spatial damping 0.9 / stiffness 700, sampled analytically and bounded to normalized presentation values. Standard and Expressive schemes are selectable. Menu/dialog previews now use the pinned Web duration curves and staged item/section fades; sheets use directional spatial springs. See [effect implementation](effects-implementation.md) for exact profiles and validation limits.

Logical state commits immediately, exit content stops receiving input, closed overlays restore an enabled trigger, reduced motion snaps transitions and busy effects stop, and idle scenes stop scheduling. Direct manipulation follows the pointer without a trailing spring. Static foundations have no invented animation.

## New composition previews

| Family | Demonstrated | Remaining scope |
| --- | --- | --- |
| [Floating action button](components/floating-action-button.md) | Small, regular and extended creation actions (the core FAB API also accepts a large size); activation count. | Full icon/shape tokens and elevation variants. |
| [Segmented selection / toggle group](components/segmented-selection-toggle-group.md) | Single-choice view and independent multi-choice formatting. | Expressive connected shapes and complete segmented-button semantics. |
| [Button group](components/button-group.md) | Grouped independent cut/copy/paste actions. | Expressive connected shape/spacing transitions. |
| [Split button](components/split-button.md) | Primary save action and separate secondary menu. | Connected visual shape and disabled-item variants. |
| [Floating action menu](components/floating-action-menu.md) | Expandable action group with reversible spatial reveal. | Exact expressive shape and stagger choreography. |
| [Text field](components/text-field.md) | Outlined single-line field, native browser editing and character count. | Native Wayland text editing/IME and complete filled/outlined tokens. |
| [Multiline text field](components/multiline-text-field.md) | Browser multiline editing, selection and IME backed by Rust UTF-8 state. | Native text layout, scrolling metrics and cross-renderer shaping parity. |
| [Search](components/search.md) | Editable query, filtered application results, empty result and clear. | Full expanded-search overlay and suggestion history. |
| [Select](components/select.md) | Profile trigger, selectable popup, arrow navigation and Escape. | Complete filled/outlined field anatomy and type-ahead. |
| [Autocomplete](components/autocomplete.md) | Editable query and filtered selectable suggestions. | Full ARIA combobox adapter and keyboard completion model. |
| [Form-field container](components/form-field-container.md) | Label, required validation, helper/error state and character count. | All adornment, counter and style variants. |
| [Date picker](components/date-picker.md) | Gregorian month navigation, correct weekdays/leap years and date selection. | Calendar keyboard grid, locale/RTL, text-entry and disabled-date policies. |
| [Date-range picker](components/date-range-picker.md) | Ordered start/end selection within a navigable month. | Cross-month ranges, locale input, disabled dates and keyboard grid. |
| [Time picker](components/time-picker.md) | Filled trigger and scrollable half-hour options in 12-hour format. | Minute-level entry, arbitrary intervals, locale formatting and complete listbox keyboard behavior. |
| [Top app bar](components/top-app-bar.md) | Application action surface with menu/search/share outcomes. | Collapsing app-bar geometry and scroll behavior. |
| [Bottom app bar](components/bottom-app-bar.md) | Action-bar composition and action outcomes. | Scaffold bottom anchoring and FAB relationship. |
| [Navigation bar](components/navigation-bar.md) | Exclusive destination selection and corresponding content. | Destination icons, badge anatomy and complete navigation tokens. |
| [Navigation rail](components/navigation-rail.md) | Destination selection and corresponding content. | Wide-layout rail placement and expressive icon/indicator geometry. |
| [Navigation drawer](components/navigation-drawer.md) | Persistent destination-list selection. | Adaptive placement and drawer-specific modal variant. |
| [Menu](components/menu.md) | Anchored menu with reference 500/150ms growth/exit, staggered items, keyboard movement and immediate exit input rejection. | Submenus, type-ahead and complete menu role adapter. |
| [Toolbar](components/toolbar.md) | Grouped application actions on a shared surface. | Density/overflow variants and roving toolbar keyboard model. |
| [Floating toolbar](components/floating-toolbar.md) | Rounded floating action surface and independent actions. | Anchored placement, collapse and drag interactions. |
| [Card](components/card.md) | Filled, outlined and shadowed surfaces with selection. | Complete elevation/interaction tokens and media anatomy. |
| [Dialog](components/dialog.md) | Modal surface/scrim with explicit Angular 150/75ms timing in the comparison, or Material Web timing with staged content/actions, dismissal, focus containment and restoration. | Full dialog semantic/assistive-technology adapter and nested ownership. |
| [Bottom sheet](components/bottom-sheet.md) | Bottom-aligned modal surface with scrim and focus lifecycle. | Drag detents, nested scroll. |
| [Side sheet](components/side-sheet.md) | Trailing modal surface with dismissal and focus lifecycle. | Adaptive standard variant. |
| [Carousel](components/carousel.md) | Manual bounded navigation with clipped spatial travel and position feedback. | Responsive sizing/masking strategy and direct drag physics. |
| [Expansion panel / accordion](components/expansion-panel-accordion.md) | Independent disclosures with rotating chevrons and reversible clipped height transitions. | Dedicated disclosure semantics, full content layout and focus-on-collapse policy. |
| [Scaffold](components/scaffold.md) | App title, destination tabs, changing content and creation action. | General responsive slot layout and inset/overlay coordination. |
| [Expressive loading indicator](components/expressive-loading-indicator.md) | Seven matched upstream polygon morphs, spring/rotation timing, determinate circle-to-soft-burst and contained/uncontained variants. | Assistive-technology announcements and hardware performance qualification. |
| [Snackbar](components/snackbar.md) | Transient archive notice, Undo, deadline and focus/hover timeout pause. | Live announcement adapter, queueing and configurable accessibility timeouts. |
| [Banner](components/banner.md) | Persistent notice with reversible reveal/fade and dismiss/view actions. | Full banner layout variants; this is a legacy/framework pattern. |
| [Data table](components/data-table.md) | Sortable sample rows with ascending, descending and unsorted header states. | General column model, multi-selection and table semantics. |
| [Sort header](components/sort-header.md) | Sort-direction toggle with reordered sample rows. | Sort announcements, multi-column data model and header semantics. |
| [Paginator](components/paginator.md) | Bounded previous/next paging with a correctly clamped range label. | Page-size selector, first/last controls and localization. |
| [Tree](components/tree.md) | Expandable nested sample rows and child selection. | Tree roles, roving arrow-key navigation and lazy nested models. |
| [Stepper](components/stepper.md) | Three-step navigation, required account entry and bounded next/back. | Nonlinear/optional steps, error presentation and complete workflow semantics. |
| [Grid list / image list](components/grid-list-image-list.md) | Four selectable application tiles in two columns. | Image resource/loading/cropping behavior and responsive tile sizing. |
| [Avatar](components/avatar.md) | Caller-owned image, centered initials and Material person fallback; circle, rounded and square; simulated image failure. | Host image loading, image transitions, complete avatar tokens and visual parity. |
| [Avatar group](components/avatar-group.md) | Mixed images, initials and fallback; MUI-style max/total counting, overlap, shape and RTL controls. | Custom surplus, per-avatar sizes and framework/device visual parity. |
| [Scrollbar](components/scrollbar.md) | Anchored interactive vertical thumb, viewport-sized track and keyboard paging; separate non-interactive vertical/horizontal Material indicator with fading and RTL. | Touch expansion, nested-scroll routing, customizable indicator tokens and device comparison. |
| [Refresh indicator](components/refresh-indicator.md) | Start, visible busy feedback, timed completion and refresh count. | Pull gesture, threshold/overscroll and official circular indicator geometry. |
| [Icon](components/icon.md) | Static presentation symbols without an external icon font. | Vector icon resource API and exact Material symbol geometry. |
| [Surface and elevation](components/surface-and-elevation.md) | Surface role/tint levels and approximate static shadows. | Physical shadow rendering and complete elevation tokens. |
| [State layer / ripple](components/state-layer-ripple.md) | Pointer-origin radial ink, rounded clipping, hold/release fade, keyboard-centered ink, cancellation and reduced motion. | Android platform RippleDrawable/sparkle shader pixel parity and hardware/device qualification. |
| [Theme tokens](components/theme-tokens.md) | Theme color-role samples with host light/dark switching. | Full tonal palette, type scale, density and dynamic-color resolution. |
| [Overlay and scrim](components/overlay-and-scrim.md) | Modal backdrop with immediate logical dismissal and focus lifecycle. | General overlay stack, nested modality and complete semantics bridge. |
| [Virtualized grid](components/virtualized-grid.md) | 100,000 independently selectable cells with bounded visible construction and scrolling. | Responsive column count, selection semantics and richer tile layout. |
| [Popover](components/popover.md) | Anchored custom-content surface with selection and Escape. | Anchor collision/flip, nested content and generic semantic ownership. |
| [Command palette](components/command-palette.md) | Editable application query, filtered commands and actual selection outcome. | Global shortcut adapter, grouped results and full keyboard-first dialog model. |
| [Application tile](components/application-tile.md) | Selectable application-name tiles. | Icon resource, status/badge, context menu and real application launching. |
| [Context-menu integration](components/context-menu-integration.md) | Right-click/ContextMenu/Shift+F10 invocation and selectable actions. | Pointer-position anchoring, submenus and complete desktop menu semantics. |
| [Shortcut hint](components/shortcut-hint.md) | Static keycap/shortcut descriptions without extra focus targets. | Keycap anatomy, platform-specific names and RTL placement. |
| [Split pane / resizable divider](components/split-pane-resizable-divider.md) | Pointer-draggable divider, arrow-key adjustment and bounded ratio. | Dedicated separator role, touch target policy and persisted layout. |
| [Breadcrumb](components/breadcrumb.md) | Three navigable path levels with selected content. | Overflow/collapse, current-page semantics and arbitrary hierarchical model. |
| [Numeric field / spin box](components/numeric-field-spin-box.md) | Bounded increment/decrement and quantity slider. | Numeric text editing/parsing, locale and spinbutton semantics. |
| [Color picker](components/color-picker.md) | Keyboard/pointer hue selection and RGB swatch. | Saturation/value, alpha, text input and color-space management. |
| [File chooser integration](components/file-chooser-integration.md) | Real browser file chooser and selected filename; no upload or file-content read. | Wayland desktop portal open/save/multi-selection adapter. |
| [Empty/error state](components/empty-error-state.md) | Empty/error/recovered states with clear-filter/retry actions. | Application error model and announcement adapter. |
| [Notification surface](components/notification-surface.md) | Notification content with reversible reveal/fade and dismiss/view actions. | System notification service, privacy, urgency, queueing and announcements. |
