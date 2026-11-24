# Bhoo component demos

Compare actual Angular Material controls with the actual libbhoo Rust core compiled to WebAssembly. The default site uses Angular Material's familiar documentation structure: component navigation, Overview / API / Examples tabs, and interactive example cards.

The catalog has Bhoo scenes include direct Angular equivalents, related Angular patterns, and explicitly unsupported Angular families. A direct equivalent identifies the family, not equal coverage. The comparison exposes current visual differences; it does not claim parity.

## Run

From the repository root, with Rust/rustup, Python 3 and Node/npm installed (Node 22.22.3+, 24.15+, or 26+):

```sh
rustup target add wasm32-unknown-unknown
python3 tools/serve-bhoo-web.py
```

Open **http://127.0.0.1:8766/**. Keep the terminal running. The server listens only on loopback and serves the site directory. Use `--port 8767` to change port, or `--no-build` to reuse the generated site. Stop with Ctrl+C.

The comparison runs locally; it does not embed the remote Angular website or fetch runtime CDN packages/fonts. First build installs exact locked npm dependencies into ignored `context/build/bhoo-angular`, then creates an approximately 3.17 MB reference bundle. No Angular CLI or framework source checkout is needed. The WASM module remains approximately 316 KB.

For the original Bhoo playground, without Node/npm dependencies:

```sh
python3 tools/serve-bhoo-web.py --rust-only
```

Open **http://127.0.0.1:8766/playground.html**. It has all Rust previews, live state, Standard/Expressive motion settings, code and specification references. Eighteen scenes use initial widget APIs; the rest are experimental compositions.

Build static files without starting the server:

```sh
python3 tools/build-bhoo-web.py
# Or only Rust and the original playground:
python3 tools/build-bhoo-web.py --rust-only
```

Serve `site` over HTTP. Opening HTML using `file://` will not load WASM correctly. After building, `context/build/bhoo-angular` may be deleted to recover space; the generated site remains usable. A subsequent comparison rebuild reinstalls the locked dependencies.

## Compare

- Choose a component and interact with each implementation independently.
- Switch between **Side by side**, adjustable **Overlay**, and **Wipe** views. Overlay/wipe are inspection modes; return to side by side for interaction.
- Both previews use the same 440 × 640 CSS px viewport. Select 360 or 600 px to inspect another size.
- Disabled, reduced motion, light/dark theme and configurable button variant settings apply to both panes. **Reset both** restores initial application state.
- **Share base colors** passes twenty resolved Angular theme colors to Bhoo's existing demo theme. Turn it off to inspect Bhoo's original palette. It does not change widget geometry, text size, alignment or motion, or supply full Material tokens.
- Pairing notes distinguish matching labels/starting values, different fixture content/layout, related patterns and missing equivalents. API displays both fixture sources; Overview links guidelines and references.

Bhoo's button corners, height, label typography/alignment and elevation currently differ visibly from Angular. Motion implementations also have different upstream origins: Angular is an additional reference, not a replacement for the documented Android/Compose/Material Web contracts. Full visual, motion and state parity remain open work.

## Verify

For an offline Delta Chat/webxdc attachment, run
`python3 tools/build-bhoo-webxdc.py --no-build` after building the full site.
The result is `context/dist/bhoo-demo.xdc`, including Angular, WASM, licenses and
corresponding source. See [webxdc packaging](../../libs/ui/libbhoo/docs/webxdc.md).

Start a compatible local Chromium with `--remote-debugging-port=9223`, then run sequentially:

```sh
node tools/test-bhoo-comparison.mjs
node tools/test-bhoo-web.mjs
```

Node must provide built-in fetch and WebSocket. `BHOO_WEB_URL` and `BHOO_CDP_PORT` override defaults. No npm test/image dependencies are required.

Comparison checks exercise both actual runtimes, trusted input, shared settings, disabled blocking, menus/dialogs, viewport alignment, all catalog desktop/mobile routes, navigation and documentation. PNG pairs, difference heatmaps and a standalone evidence report are saved under ignored `context/bhoo-comparison`. Baselines cover every direct equivalent, with additional interaction-state captures. Whole-pane pixel differences include fixture layout, text metrics and rasterization; they are not component match scores or phase-aligned animation validation.

The original explorer checks actual Rust pointer/keyboard state, editing/file adapters, disabled/reduced states, virtualization, idle scheduling and all catalog routes. See [comparison decisions and limitations](../../libs/ui/libbhoo/docs/angular-comparison.md).

## Implementation and license

The Rust crate depends only on libbhoo with native features disabled. JavaScript paints Rust draw/semantics commands using Canvas 2D; native rendering remains Wayland/EGL/GLES. Browser editing/IME uses bounded HTML adapters, and file selection reads only a filename. Native shaping/IME, canvas assistive technology integration and complete component APIs remain unfinished. See [per-family coverage](../../libs/ui/libbhoo/docs/demo-coverage.md).

Bhoo and the independently written comparison host/fixtures are GPL-3.0-only. The Angular reference has its own MIT/Apache-2.0/0BSD notices. Build output includes the actual runtime inventory and complete license texts under `site/angular`, plus the GPL license, root notices and compressed corresponding source. Preserve these when distributing the site. Angular/Google do not endorse this independent comparison.
