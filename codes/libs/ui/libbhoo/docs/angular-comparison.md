# Angular Material comparison lab

Status: implemented as a development tool, not a statement of component parity.

## Purpose

Use real Angular Material controls as an independent visual reference for the actual Bhoo Rust/WASM renderer. The default browser entry point now follows the familiar Angular Material documentation organization: component navigation, Overview / API / Examples tabs, and bordered interactive example cards. This is independently written project UI, not a copy of Angular's branding or an upstream-hosted page.

The original Bhoo explorer remains at `playground.html`. All 77 Bhoo scenes remain available. The comparison catalog labels 41 direct Angular equivalents, 11 related Angular compositions/patterns, and 25 families with no direct Angular equivalent. A direct equivalent identifies the component family; it does not claim equal implementation scope. Related references and missing equivalents are explicitly marked.

Official references:

- [Angular Material component catalog](https://material.angular.dev/components/categories)
- [Button overview and examples](https://material.angular.dev/components/button/overview)
- [Angular Material theming](https://material.angular.dev/guide/theming)
- [Angular component source and license](https://github.com/angular/components)
- [Angular documentation component-viewer organization](https://github.com/angular/material.angular.io/tree/main/src/app/pages/component-viewer)

## Actual reference, isolated runtime

`codes/apps/bhoo-web/angular-reference` contains a pinned package manifest, npm lockfile, reference templates and a Sass theme. Angular Material/CDK are 22.2.6; Angular core/common/compiler/forms/platform-browser are 22.2.5. RxJS is 7.8.2. The lockfile pins transitive dependencies and integrity hashes. These versions are separate from the earlier source-only research snapshot; the demo labels the actual runtime version.

`tools/build-bhoo-angular.mjs` installs only into ignored `context/build/bhoo-angular`, bundles with esbuild and compiles the official Sass theme. There is no Angular CLI, npm project in the Rust crate, framework checkout, external font request or runtime CDN. Angular's compiler is included for these small JIT-compiled fixtures. The reference bundle is approximately 3.17 MB uncompressed; the Rust module remains approximately 316 KB. The plain Bhoo playground does not load the Angular bundle.

The Angular reference and Bhoo renderer run in separate same-origin iframes. The reference uses actual Angular Material modules and behaviors. Fixture CSS sets outer arrangement, padding and gaps, without restyling component internals to resemble Bhoo. Bhoo remains the original renderer and widget implementation. Message handling validates origin, parent/source and a generation number so late messages from replaced fixtures cannot update the current pair.

The Angular fixture source is visible under API. The Rust example and existing specification/reference links are available under API and Overview. The corresponding-source archive includes the new host, reference source, lockfile, build tools and visual test tools.

## Fair comparison controls

- Default viewport: **440 × 640 CSS px**, rendered at 100% scale in both panes. Width may be changed to 360 or 600 px. Small screens scroll within the preview area; the page itself does not gain horizontal overflow.
- Side by side supports independent real pointer, touch and keyboard interaction in each pane. It does not mirror a click to a potentially different target in the other implementation.
- Overlay blends the Angular viewport over the Bhoo viewport, with adjustable opacity. Wipe clips Angular at an adjustable vertical boundary. Both modes prevent pointer/keyboard interaction with overlapping content; return to side by side to interact.
- Reset both recreates both runtimes and restores initial application state, while retaining shared preview settings.
- Disabled, reduced motion, theme and the configurable button variant are sent to both implementations. Changing settings does not restart the example.
- **Share base colors** reads twenty resolved colors from the live Angular theme and supplies those values to Bhoo's demo-only palette bridge: surface, on-surface, surface-container, surface-container-high, surface-container-highest, primary, on-primary, secondary-container, outline, error, container, inverse and variant foreground/background roles. This maps onto Bhoo's existing simplified theme; it does not implement the entire Material token system. Native defaults and normal playground colors are unchanged. Turn it off to inspect Bhoo's original palette.
- Both comparison panes request Arial with a sans-serif fallback. Bhoo uses per-command size, weight, alignment and tracking with backend-provided ASCII advances. Unicode shaping and native font weight rendering still require review. Missing system fonts may affect screenshots.
- Reduced-motion fixtures stop/snap movement. For Angular's indeterminate progress examples the fixture presents a static determinate indicator; global ripple and CSS motion are disabled. This is a comparison-fixture accommodation, not a claim about upstream Angular defaults. Dialog duration inputs are reduced to zero. With full motion, the reference preserves Angular's own ripple durations (225 ms enter / 150 ms exit), dialog defaults and component timing; it is not retimed to imitate Bhoo. Other animations still require individual review.
- Hidden documentation previews pause Bhoo scheduling. CSS animations in the reference are paused while its preview is hidden.

## Fixture validity and known mismatches

Some fixtures share labels and initial values, including button variants, basic selection controls, sliders, menus and dialogs. Others compare an existing Bhoo composition with an independent Angular example; their layout/content differences are labeled. No whole-pane comparison can certify an individual widget in those cases.

The Rust widgets now implement 40 px pill buttons, per-component typography, three-layer elevation shadows, vector checkbox/switch indicators, matching slider and tab geometry, outlined/filled field containers, and the reference's baseline card, table, drawer, stepper and calendar arrangements. Shared fixture constraints fix the virtual list's inconsistent item size and prevent the drawer from overflowing its preview. Chip-set outer margins are compensated in the fixture layout without changing chip internals.

The comparison selects an explicit Angular Material ink profile (225 ms growth/minimum feedback, 150 ms fade, fixed pointer origin and solid radial ink). The original Material Web profile remains available and is the standalone/native default. Classic MDC progress timelines are separate APIs from the existing Compose Material 3 progress variants. Angular menu/dialog profiles use their own duration curves; this does not make every other overlay animation equivalent.

Remaining work includes Unicode line wrapping/shaping, exact native medium-weight fonts and shadow sampling, open picker/select/autocomplete geometry and full keyboard behavior, completed stepper icons, calendar boundary navigation/current-date integration, and deterministic phase-aligned motion comparisons. Baseline screenshots alone cannot establish equality for these states. The current calendar fixture starts in November 2026; it is not a general current-date service.

Angular's available components also do not imply coverage of every Material 3 Expressive family. Loading morphs, floating action menus, desktop shell integrations and other unsupported families must be reviewed against their relevant Android/Compose/Material Web references instead of substituting unrelated Angular widgets.

## Visual evidence and checks

Start a compatible local Chromium with remote debugging port 9223, then run:

```sh
node tools/test-bhoo-comparison.mjs
```

`BHOO_WEB_URL` and `BHOO_CDP_PORT` override the local defaults. Tests use Node built-ins and CDP; no Playwright/npm test dependencies or image libraries are installed.

The checks exercise both actual runtimes, trusted clicks and keyboard events, shared settings, light/dark surface pixels, independent activation, disabled blocking, checked/toggled state, menu/dialog open and Escape, equal viewport dimensions, overlay/wipe registration, documentation visibility, and all 77 desktop/mobile routes. All runtime requests must remain on the local origin.

Artifacts go under ignored `context/bhoo-comparison`: paired PNGs, RGB difference heatmaps, `results.json`, a standalone `index.html` evidence report, and desktop/mobile screenshots. Baseline pairs are captured for every direct equivalent with shared light colors and reduced motion. Additional captures show hover, pressed ink, focus, disabled, dark, checked, menu-open and dialog-open states. Hover and pressed states are captured sequentially on the two real implementations; they are not phase-aligned animation measurements.

The PNG decoder/encoder/difference code uses only Node zlib. Chromium viewport screenshots are cropped after capture so a beyond-viewport screenshot cannot change scrollbar width and move centered fixtures. It counts pixels whose maximum RGB channel difference exceeds 16/255, and includes mean absolute channel delta. Counts include layout, glyph rasterization and fixture differences. The report explicitly avoids interpreting the percentage as a component match score. No acceptance threshold or certification is silently inferred.

Still required before a parity claim: aligned canonical fixtures for all variants, a complete visual state matrix (including applicable validation/error, drag, mixed, selected and focus states), deterministic clocks and phase-aligned motion recordings, token/geometry/type specification review, font control, accessibility review, and Wayland/GLES validation. Passing the lab's functional checks does not establish these.

## Verification record (2026-11-09)

- 52 core Rust tests, including zero allocations for warmed unchanged controls, bounded classic progress and Angular quick-tap lifetime tests.
- Two browser-gallery Rust tests covering calendar math and every registered scene's enabled/disabled construction.
- Native and WebAssembly Clippy with warnings denied.
- 26 standalone browser integration groups; 19 comparison integration groups and 51 paired image captures.
- Ten isolated Wayland/GLES smoke captures across normal/reduced motion and display scales one/two.

The subsequent chip fix adds 30 phase-aligned samples against the genuine
Angular CSS transitions (selection, deselection, delayed stroke, interrupted
reversal and hover). Run `node tools/test-bhoo-chip-motion.mjs` with the same
CDP setup. This verifies chip geometry/timing within stated test tolerances;
it does not certify
SVG edge rasterization. Two additional Rust chip lifecycle tests pass, and
the allocation test now includes animated, repeatedly reversed chip selection.

The menu follow-up adds dedicated transparent menu rows and menu roles, matching
reference typography/padding and preserving keyboard opening origin. Fifteen
phase-aligned menu samples cover enter/exit geometry, opacity and typography;
interrupted-entry teardown and reduced motion are checked separately. Run
`node tools/test-bhoo-menu-motion.mjs`. Entry pointer blocking, keyboard targets,
disabled rows and focus restoration also have Rust regression tests. See the
[menu record](components/menu.md) for remaining scope and compositing limits.

The input follow-up compares 63 phase-aligned samples with the pinned Angular
runtime using `node tools/test-bhoo-input-motion.mjs`. It covers outlined and
filled labels, focus and blur, interrupted transitions, required validation,
the filled underline, nonempty values and reduced motion. The geometry checks
allow small font-baseline differences; they do not establish pixel parity,
complete input variant coverage, RTL, IME behavior or native backend parity.

These checks establish the exercised behavior, not comprehensive state or pixel parity. The visual evidence and before-change snapshots remain under ignored `context`.

The checkbox comparison offers **Link checkbox states (Angular → Bhoo)**,
enabled by default. Angular's committed checked/mixed values and focus state
are passed to the Bhoo fixture so they can be seen at the same time. Turn it
off to operate both controls independently. This is a comparison fixture; it
does not synchronize pointer ripples or replace the Rust checkbox renderer.

The expansion-panel follow-up compares 27 phase-aligned samples for a middle
panel, adjacent expanded panels, closing and a shortened reversal. A separate
browser capture checks the expanded header surface and layout. Run
`node tools/test-bhoo-expansion-motion.mjs` and
`node tools/test-bhoo-expansion-layout.mjs`.

## Storage, build and attribution

Build the complete local site with `python3 tools/build-bhoo-web.py` and serve it with `python3 tools/serve-bhoo-web.py --no-build`. For a dependency-free Rust playground build use `--rust-only` and open `/playground.html`. The server binds loopback only.

The reference build produces `site/angular/reference.js`, `theme.css`, an actual bundled dependency/version inventory, esbuild legal comments and `THIRD_PARTY_LICENSES.txt`. These generated files are ignored. The build refuses a bundled package without a license file. Angular packages are MIT, RxJS is Apache-2.0 and tslib is 0BSD; original copyright/license texts are distributed alongside the reference. Bhoo and its independently written host/fixtures remain GPL-3.0-only. See [root notices](../../../../../THIRD_PARTY_NOTICES.md).

There are no framework source clones or font/image collections in this build. After building, the temporary npm installation/cache may be deleted from `context/build/bhoo-angular`; the already built site continues working. Rebuilding reinstalls exact locked dependencies. Browser test binaries/profiles and intermediate Rust test output may likewise be removed after verification.

## Catalog mapping

This maps component families to the pinned Angular reference; no row certifies visual or state parity.

| Bhoo family | Relationship | Angular reference |
| --- | --- | --- |
| Button | Direct equivalent | [button](https://material.angular.dev/components/button/overview) |
| Icon button | Direct equivalent | [icon-button](https://material.angular.dev/components/button/overview) |
| Floating action button | Direct equivalent | [floating-action-button](https://material.angular.dev/components/button/overview) |
| Segmented selection / toggle group | Direct equivalent | [segmented-selection-toggle-group](https://material.angular.dev/components/button-toggle/overview) |
| Button group | Related pattern | [button-group](https://material.angular.dev/components/button/overview) |
| Split button | No equivalent | — |
| Floating action menu | No equivalent | — |
| Checkbox | Direct equivalent | [checkbox](https://material.angular.dev/components/checkbox/overview) |
| Radio group | Direct equivalent | [radio-group](https://material.angular.dev/components/radio/overview) |
| Switch | Direct equivalent | [switch](https://material.angular.dev/components/slide-toggle/overview) |
| Chip | Direct equivalent | [chip](https://material.angular.dev/components/chips/overview) |
| Slider | Direct equivalent | [slider](https://material.angular.dev/components/slider/overview) |
| Range slider | Direct equivalent | [range-slider](https://material.angular.dev/components/slider/overview) |
| Text field | Direct equivalent | [text-field](https://material.angular.dev/components/input/overview) |
| Multiline text field | Direct equivalent | [multiline-text-field](https://material.angular.dev/components/input/overview) |
| Search | Related pattern | [search](https://material.angular.dev/components/input/overview) |
| Select | Direct equivalent | [select](https://material.angular.dev/components/select/overview) |
| Autocomplete | Direct equivalent | [autocomplete](https://material.angular.dev/components/autocomplete/overview) |
| Form-field container | Direct equivalent | [form-field-container](https://material.angular.dev/components/form-field/overview) |
| Date picker | Direct equivalent | [date-picker](https://material.angular.dev/components/datepicker/overview) |
| Date-range picker | Direct equivalent | [date-range-picker](https://material.angular.dev/components/datepicker/overview) |
| Time picker | Direct equivalent | [time-picker](https://material.angular.dev/components/timepicker/overview) |
| Top app bar | Related pattern | [toolbar](https://material.angular.dev/components/toolbar/overview) |
| Bottom app bar | Related pattern | [toolbar](https://material.angular.dev/components/toolbar/overview) |
| Navigation bar | Related pattern | [tabs](https://material.angular.dev/components/tabs/overview) |
| Navigation rail | Related pattern | [navigation-drawer](https://material.angular.dev/components/sidenav/overview) |
| Navigation drawer | Direct equivalent | [navigation-drawer](https://material.angular.dev/components/sidenav/overview) |
| Tabs | Direct equivalent | [tabs](https://material.angular.dev/components/tabs/overview) |
| Menu | Direct equivalent | [menu](https://material.angular.dev/components/menu/overview) |
| Toolbar | Direct equivalent | [toolbar](https://material.angular.dev/components/toolbar/overview) |
| Floating toolbar | Related pattern | [toolbar](https://material.angular.dev/components/toolbar/overview) |
| Card | Direct equivalent | [card](https://material.angular.dev/components/card/overview) |
| Dialog | Direct equivalent | [dialog](https://material.angular.dev/components/dialog/overview) |
| Bottom sheet | Direct equivalent | [bottom-sheet](https://material.angular.dev/components/bottom-sheet/overview) |
| Side sheet | No equivalent | — |
| Divider | Direct equivalent | [divider](https://material.angular.dev/components/divider/overview) |
| List and list item | Direct equivalent | [list-and-list-item](https://material.angular.dev/components/list/overview) |
| Carousel | No equivalent | — |
| Expansion panel / accordion | Direct equivalent | [expansion-panel-accordion](https://material.angular.dev/components/expansion/overview) |
| Scaffold | No equivalent | — |
| Badge | Direct equivalent | [badge](https://material.angular.dev/components/badge/overview) |
| Progress indicator | Direct equivalent | [progress-indicator](https://material.angular.dev/components/progress-bar/overview) |
| Expressive loading indicator | No equivalent | — |
| Snackbar | Direct equivalent | [snackbar](https://material.angular.dev/components/snackbar/overview) |
| Tooltip | Direct equivalent | [tooltip](https://material.angular.dev/components/tooltip/overview) |
| Banner | No equivalent | — |
| Data table | Direct equivalent | [data-table](https://material.angular.dev/components/table/overview) |
| Sort header | Related pattern | [data-table](https://material.angular.dev/components/sort/overview) |
| Paginator | Direct equivalent | [paginator](https://material.angular.dev/components/paginator/overview) |
| Tree | Direct equivalent | [tree](https://material.angular.dev/components/tree/overview) |
| Stepper | Direct equivalent | [stepper](https://material.angular.dev/components/stepper/overview) |
| Grid list / image list | Direct equivalent | [grid-list-image-list](https://material.angular.dev/components/grid-list/overview) |
| Avatar | No equivalent | — |
| Scrollbar | No equivalent | — |
| Refresh indicator | No equivalent | — |
| Typography / text | Direct equivalent | [typography-text](https://material.angular.dev/components/typography/overview) |
| Icon | No equivalent | — |
| Surface and elevation | Related pattern | [card](https://material.angular.dev/components/card/overview) |
| State layer / ripple | Direct equivalent | [state-layer-ripple](https://material.angular.dev/components/ripple/overview) |
| Focus ring | Direct equivalent | [focus-ring](https://material.angular.dev/components/button/overview) |
| Theme tokens | No equivalent | — |
| Overlay and scrim | Related pattern | [dialog](https://material.angular.dev/components/dialog/overview) |
| Virtualized list | Direct equivalent | [virtualized-list](https://material.angular.dev/cdk/scrolling/overview) |
| Virtualized grid | No equivalent | — |
| Scroll area / viewport | No equivalent | — |
| Popover | No equivalent | — |
| Command palette | No equivalent | — |
| Application tile | No equivalent | — |
| Context-menu integration | Related pattern | [menu](https://material.angular.dev/components/menu/overview) |
| Shortcut hint | No equivalent | — |
| Split pane / resizable divider | No equivalent | — |
| Breadcrumb | No equivalent | — |
| Numeric field / spin box | No equivalent | — |
| Color picker | No equivalent | — |
| File chooser integration | No equivalent | — |
| Empty/error state | No equivalent | — |
| Notification surface | No equivalent | — |
