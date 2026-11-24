# Offline webxdc demo

The Bhoo/Angular comparison can be sent as a single `.xdc` attachment and opened
inside a messenger supporting webxdc. It includes the real Rust/WASM renderer,
the pinned Angular reference, all 77 catalog routes, runtime licenses and the
compressed corresponding source. It makes no runtime CDN or font requests.

```sh
python3 tools/build-bhoo-webxdc.py
# Reuse an already built site:
python3 tools/build-bhoo-webxdc.py --no-build
```

The output is `context/dist/bhoo-demo.xdc`. This is a Deflate ZIP with root
`index.html` and `manifest.toml`, following the
[webxdc container specification](https://webxdc.org/docs/spec/format.html).
Fixed ZIP metadata makes packaging reproducible for identical built inputs.
The messenger supplies its own `webxdc.js`; none is bundled or overridden.
Frame messages also handle opaque webview origins: the target origin becomes
`*` only when `location.origin` is `null`; receivers still check the source
window, message namespace and current run identifier.

This is a local visual inspection tool: control state is independent on each
device, and it sends no chat status updates or realtime data. It needs a modern
WebView with JavaScript modules, WebAssembly, Canvas 2D and iframe support.
External guideline links require the messenger's normal confirmation and network
access outside the app; they are not needed to run the examples. Source downloads
are included, although support for the HTML download attribute varies by client.
The optional file chooser still only records a chosen filename.

The [messenger constraints](https://webxdc.org/docs/spec/messenger.html) require
all app resources to be local and Internet access to be denied. Packaging does
not certify every mobile WebView or resolve the visual gaps recorded in the
[comparison TDD](angular-comparison.md).

Delivery account credentials and invitations are kept only under ignored
`context/webxdc-delivery`, separate from source and from existing user profiles.
They must never be added to the archive or the corresponding-source download.

## Continuous Beer CSS-inspired showcase

Build with `python3 tools/build-bhoo-web.py --rust-only`, then package with `python3 tools/build-bhoo-webxdc.py --no-build --gallery --output context/dist/bhoo-showcase.xdc`. This gallery includes all 97 current component examples on one continuous page, using the navigation order and page structure of the MIT-licensed Beer CSS source. The expanded/collapsed rail, floating toolbar, yellow introduction, section and component headings, source viewer and mobile navigation are rendered in Rust through Bhoo's WASM command stream. The host provides layout, document scrolling, Canvas painting, native text input, frame messaging and visibility lifecycle. Beer CSS, Vue and Angular runtimes are absent from this package.

The host keeps nearby examples mounted and unloads distant frames to bound memory; an unloaded example's local interaction state resets when revisited. This is a gallery of the existing experimental implementations, not a claim of complete component parity. Use `node tools/test-bhoo-one-page.mjs` against the unpacked gallery to exercise all examples, offline requests, Rust navigation, code viewer, theme and mobile menu. The rendered shell is implemented in `codes/apps/bhoo-web/src/showcase.rs`.

### Official Material Symbols

The continuous gallery now paints its menu, widgets, palette, motion, theme, download, scroll and code controls using official Google Material Symbols Outlined. The Bhoo wordmark remains branding. The icon example displays the 20 bundled symbols, and icon buttons use `Frame::material_icon_button` with separate accessible labels. Apps can draw decorative symbols using `Frame::paint_material_icon(bounds, MaterialIcon::Palette, color)`. This API preserves the square design box and uses the shared vector geometry backend on native and WASM.

Sources and the Apache-2.0 license are retained in `codes/libs/ui/libbhoo/assets/material-symbols`; `SOURCE.json` pins the upstream revision. Run `python3 tools/generate-material-symbols.py` to regenerate meshes. Only the used subset is bundled to keep storage small; this is not the entire Google Fonts icon catalog.

## Single-canvas continuous showcase

Build the site with `python3 tools/build-bhoo-web.py --rust-only`, then package with `python3 tools/build-bhoo-webxdc.py --no-build --canvas-gallery --output context/dist/bhoo-single-canvas.xdc`. Open `canvas.html` during local development.

This version has exactly one visible canvas, no iframes, and one WebAssembly instance. `src/onepage.rs` owns the responsive page layout, document offset, bounds, section navigation, touch drag threshold, click cancellation, inertia, keyboard scrolling and scrollbar dragging. The browser forwards wheel/pointer/keyboard events and paints Bhoo packets into regions of the same canvas. The backing canvas stays viewport sized, capped at 2× device pixel ratio; the logical document can be arbitrarily tall without allocating a document-sized bitmap.

Each example retains a separate Rust `Demo` context inside that single WASM instance. Only visible regions are painted, but component values survive scrolling offscreen. The fixed rail, toolbar, hero, section headings, examples, source viewer and mobile drawer all use the same canvas. The browser creates a temporary transparent native input/textarea only for text entry and IME; it does not render a second component gallery.

Wheel and trackpad scrolling move the document except over explicit nested scroll examples, navigation, source code or open component overlays. Touch dragging moves the document with inertia and cancels the pressed example once the drag exceeds 6 logical pixels; slider and text entry gestures stay with the widget. Page Up, Page Down, Home and End scroll the document. Arrow keys scroll when no widget owns keyboard focus. Pointer cancellation, window blur and resize cancel ongoing gestures. The right-edge canvas scrollbar can be dragged.

Run `node tools/test-bhoo-single-canvas.mjs` against the local site. To test an extracted package, set `BHOO_GALLERY_URL` to its server origin and `BHOO_GALLERY_ENTRY=/`. The check visits all 97 examples, verifies one canvas/zero iframes/one WASM download, exercises wheel, keyboard and touch scrolling, checks checkbox and text state after scrolling away and back, opens/closes the canvas source viewer and mobile drawer, and rejects external requests or browser errors. Rust unit tests cover scroll bounds, keyboard navigation, drag cancellation, inertia and widget gesture ownership.

### Canvas dialog and shell motion

Source-code buttons use the existing Bhoo `OverlayState` with `OverlayKind::Dialog` and `Frame::overlay`; the host no longer substitutes a plain rectangular source panel or draws its scrim. The dialog sizes to its content up to 76% of viewport height, supports source scrolling, traps canvas keyboard navigation, dismisses through Close, Escape or its scrim, keeps exit frames alive and restores the trigger's focus. It remains on the same canvas.

Desktop navigation width uses an interruptible 300ms (medium2) standard easing transition. Theme and palette colors use a separate 200ms (short4) standard easing transition, interpolated in Rust from the currently presented palette. Both use `cubic-bezier(0.2, 0, 0, 1)` and snap under reduced motion. These are chosen token pairings for this utility-focused showcase, not a claim that Material mandates these durations for every rail or theme switch. Palette and theme buttons have distinct actions.

References: [requested easing and duration guidance](https://m3.material.io/styles/motion/easing-and-duration/applying-easing-and-duration), [official MDC Android easing and duration tokens](https://github.com/material-components/material-components-android/blob/master/docs/theming/Motion.md#curves-easing--duration). The linked M3 page requires JavaScript; the official Android source supplies the standard curve and duration-token values. Browser checks verify intermediate rail widths and palette colors, settled endpoints, dialog close/scrim/Escape and retained single-canvas behavior. Rust tests verify continuous reversal and reduced-motion endpoints.

### Scroll smoothness and rendering budget

Page and nested wheel deltas are accumulated and approached using a time-based exponential response (18/s), with the remaining fraction applied exactly at rest. Touch dragging remains direct; its release inertia is unchanged. Reduced motion applies wheel deltas immediately. New drags, navigation and cancellation discard pending page motion. Pointer and wheel events of the same kind are coalesced before rendering to avoid an input backlog.

Settled visible surfaces reuse an OffscreenCanvas raster tile instead of rebuilding Rust commands and replaying vector/text painting every frame. There is still one visible canvas and one WASM instance. Tiles are invalidated for input, size, palette, focus, active animations, timer deadlines and rail selection changes. Offscreen tiles are discarded and the tile budget is capped at 24 MiB; browsers without OffscreenCanvas use direct rendering. Raster caching pauses while the rail changes width, avoiding repeated large allocations during layout animation. No disk image cache is created.

`node tools/profile-bhoo-canvas-scroll.mjs` compares the same desktop scroll interaction with caching disabled/enabled. In the local 1440×1000, DPR 1 run, median CPU rendering work fell from 2.4ms to 0.6ms and p95 from 3.5ms to 1.3ms; median frame interval was 16.7ms in both. Cache usage was about 6.6 MiB. These are local browser measurements, not an FPS guarantee for other hardware or every component. Raw results are saved under ignored `context/bhoo-single-canvas/scroll-performance.json`.
