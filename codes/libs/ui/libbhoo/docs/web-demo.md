# Browser component explorer

Status: implemented as a local demo; 77 live previews and catalog specification pages. Eighteen scenes use initial widget APIs and 59 are experimental compositions. See [per-family coverage](demo-coverage.md).

## Purpose and references

Use the explorer to review current controls, state transitions and motion before integrating them into SpectreOS applications. The organization follows the useful patterns of the [CUI browser demo](https://yappologistic.github.io/CUI/) and [Material component documentation](https://m3.material.io/components): searchable component navigation, interactive examples, state/motion notes, code and reference tabs. No upstream page source or artwork is copied.

## Decisions

- Compile the existing Rust core to WebAssembly, with no default native dependencies or JavaScript widget replicas.
- Present its draw commands through Canvas 2D; keep Wayland/EGL/GLES as the native backend. Browser and native text rendering are different and visual parity is not yet asserted.
- Preserve pointer/key input edges in a queue. Export semantics for inspection, but do not claim screen-reader support for canvas controls.
- Animate only while effects are active, input is pending, or a semantic deadline is due. Pause hidden previews and honor reduced motion.
- Generate the catalog from the English component specifications. Label experimental compositions and their remaining scope explicitly.
- Keep the Rust playground free of npm dependencies, bundled fonts and framework assets. The optimized WASM module is approximately 316 KB. The optional comparison entry point locally bundles pinned Angular Material, with isolated build dependencies under ignored context and complete runtime attribution. Supply a corresponding-source archive with license downloads.

## Validation and gaps

The CDP integration test checks all 77 desktop/mobile scenes and disabled controls, actual Rust control state, browser editing/file adapters, modal focus/dismissal, menu keys, step validation, sorting, reduced/reversible motion, virtual scrolling, idle scheduling, documentation tabs and search. It saves desktop and mobile screenshots. This does not establish complete Material state/motion parity, touch-device coverage, accessibility conformance or native GPU performance.

See [run/build instructions](../../../codes/apps/bhoo-web/README.md), [implementation status](implementation-progress.md), and [license review](licenses-and-attribution.md). Remaining toolkit limitations apply equally to the web demo.

Browser editable fields use an HTML input/textarea adapter for selection and IME, committing bounded UTF-8 into Rust state. File selection opens the real browser file chooser and reads only the filename. These are browser adapters, not completed native text or desktop-portal implementations.

The inspector selects Standard/Expressive spatial motion. Radial ink, seven-shape loading, circular/linear progress and typed menu/dialog choreography are shared Rust implementations; see [effect provenance and validation](effects-implementation.md).

The default entry point is now the [Angular comparison lab](angular-comparison.md), with 41 direct Angular equivalents, 11 related patterns and 25 explicit missing equivalents. The original explorer is at `/playground.html`. Shared controls, equal viewport dimensions, overlay/wipe inspection, and saved visual evidence support comparison; they do not establish parity.
