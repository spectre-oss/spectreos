// Page definitions: navigation, authored content and source-document composition.
import fs from 'node:fs';
import path from 'node:path';
import { renderMarkdown, extractSection, escapeHtml, codeBlockHtml, slugify } from './markdown.mjs';
import {
  COMPONENTS, LANDING_DIR, REPO_URL, SHIRAZ_URL, REPO_BRANCH, RELEASES_API,
  readRepoFile, readVersion, libbhooPublishable, makeLinkResolver, makeImageResolver,
} from './repo.mjs';
import { logoSvg } from './layout.mjs';

const content = (name) => fs.readFileSync(path.join(LANDING_DIR, 'src/content', name), 'utf8');

/** Source document -> site location, used to rewrite relative links inside rendered docs. */
export const LINK_MAP = new Map([
  ['README.md', 'docs/'],
  ['SECURITY.md', 'docs/security/'],
  ['codes/libs/ui/libbhoo/README.md', 'docs/libbhoo/'],
  ['codes/libs/ui/libbhoo/DEVELOPMENT.md', 'docs/libbhoo/#development-guide'],
  ['codes/apps/README.md', 'docs/apps/'],
  ['codes/apps/wisp/README.md', 'docs/wisp/'],
  ['codes/apps/keyboard/README.md', 'docs/apps/#keyboard'],
  ['codes/apps/settings/README.md', 'docs/apps/#settings'],
  ['codes/apps/files/README.md', 'docs/apps/#files'],
  ['codes/apps/calendar/README.md', 'docs/apps/#calendar'],
  ['codes/apps/bhoo-web/README.md', 'docs/apps/#bhoo-web-demos'],
  ['codes/os/specrte-nix-f1/README.md', 'docs/spectref1/'],
  ['docs/writing-spectreos-apps/README.md', 'docs/writing-apps/'],
  ['docs/writing-spectreos-apps/app-structure.md', 'docs/writing-apps/app-structure/'],
  ['docs/writing-spectreos-apps/cargo-commands.md', 'docs/writing-apps/cargo-commands/'],
  ['docs/writing-spectreos-apps/web-debugging.md', 'docs/writing-apps/web-debugging/'],
  ['docs/writing-spectreos-apps/responsiveness.md', 'docs/writing-apps/responsiveness/'],
]);

export const NAV = [
  {
    title: 'Start',
    items: [
      { id: 'docs', title: 'Overview', url: 'docs/' },
      { id: 'getting-started', title: 'Getting started', url: 'docs/getting-started/' },
    ],
  },
  {
    title: 'Components',
    items: [
      { id: 'libbhoo', title: 'libbhoo UI toolkit', url: 'docs/libbhoo/' },
      { id: 'wisp', title: 'Wisp compositor', url: 'docs/wisp/' },
      { id: 'apps', title: 'Native apps', url: 'docs/apps/' },
      { id: 'spectref1', title: 'Spectre NixOS · Poco F1', url: 'docs/spectref1/' },
    ],
  },
  {
    title: 'Build apps',
    items: [
      {
        id: 'writing-apps', title: 'Writing apps', url: 'docs/writing-apps/',
        children: [
          { id: 'app-structure', title: 'App structure', url: 'docs/writing-apps/app-structure/' },
          { id: 'cargo-commands', title: 'Cargo commands', url: 'docs/writing-apps/cargo-commands/' },
          { id: 'web-debugging', title: 'Web debugging', url: 'docs/writing-apps/web-debugging/' },
          { id: 'responsiveness', title: 'Responsiveness', url: 'docs/writing-apps/responsiveness/' },
        ],
      },
    ],
  },
  {
    title: 'Project',
    items: [
      { id: 'releases', title: 'Releases & versioning', url: 'docs/releases/' },
      { id: 'security', title: 'Security & licensing', url: 'docs/security/' },
    ],
  },
];

/** Flattened reading order for previous/next links. */
export function flatNav() {
  const out = [];
  for (const g of NAV)
    for (const it of g.items) {
      out.push(it);
      for (const c of it.children || []) out.push(c);
    }
  return out;
}

// ---------------------------------------------------------------------------

export class PageBuilder {
  constructor(env) {
    this.env = env;
    this.parts = [];
    this.headings = [];
    this.usedIds = new Map();
  }
  md(src, { from = '', ...opts } = {}) {
    const { html } = renderMarkdown(src, {
      resolveLink: makeLinkResolver(from, this.env),
      resolveImage: makeImageResolver(from),
      usedIds: this.usedIds,
      headings: this.headings,
      ...opts,
    });
    this.parts.push(html);
    return this;
  }
  file(rel, { section, exclude, includeHeading = false, ...opts } = {}) {
    let src = readRepoFile(rel);
    if (section) src = extractSection(src, section, { includeHeading, exclude });
    return this.md(src, { from: rel, ...opts });
  }
  html(h) {
    this.parts.push(h);
    return this;
  }
  toString() {
    return this.parts.join('\n');
  }
}

const head = ({ eyebrow, title, lede }) =>
  `<header class="page-head"><p class="eyebrow">${escapeHtml(eyebrow)}</p><h1>${escapeHtml(title)}</h1>${lede ? `<p class="lede">${escapeHtml(lede)}</p>` : ''}</header>`;

const KIND_LABEL = { crate: 'crate', app: 'app', 'embedded-app': 'embedded app', os: 'OS' };

function componentsData(env) {
  return COMPONENTS.map((c) => ({
    id: c.id, name: c.name, kind: c.kind, tagPrefix: c.tagPrefix, version: readVersion(c),
    binary: c.binary || null, artifact: c.artifact,
  }));
}

function componentsTable(env) {
  const rows = COMPONENTS.map((c) => {
    const v = readVersion(c);
    const docUrl = env.base + c.doc;
    return `<tr><td><a href="${docUrl}"><strong>${escapeHtml(c.name)}</strong></a><br><span class="kind kind-${c.kind}">${KIND_LABEL[c.kind]}</span></td>
<td>${v ? `<code>${v}</code>` : '<span class="muted">n/a</span>'}</td>
<td><code>${c.tagPrefix}&lt;X.Y.Z&gt;</code></td>
<td>${escapeHtml(c.artifact)}${c.binary ? `<br><code>${c.binary}-linux-x86_64</code>` : ''}</td>
<td data-live-for="${c.id}" class="live-cell"><span class="muted">Checking…</span></td></tr>`;
  }).join('\n');
  return `<div class="table-wrap"><table class="components-table"><thead><tr><th>Component</th><th>Source version</th><th>Release tag</th><th>Published artifact</th><th>Latest release</th></tr></thead><tbody>${rows}</tbody></table></div>`;
}

function dataScript(env) {
  const data = { api: RELEASES_API, repo: REPO_URL, components: componentsData(env) };
  return `<script type="application/json" id="spectreos-data">${JSON.stringify(data).replace(/</g, '\\u003c')}</script>`;
}

function liveReleasesWidget(env) {
  return `<section class="live-releases" id="live-releases-widget" aria-labelledby="live-releases-title">
<div class="live-head"><div><h3 id="live-releases-title" class="plain">Live GitHub releases</h3><p class="muted">Fetched in your browser from the GitHub API. Download buttons link straight to each release asset.</p></div>
<div class="live-controls"><label class="sr-only" for="release-filter">Filter by component</label><select id="release-filter" class="select"><option value="">All components</option>${COMPONENTS.map((c) => `<option value="${c.id}">${escapeHtml(c.name)}</option>`).join('')}</select></div></div>
<div id="release-list" class="release-list" aria-live="polite"><p class="muted">Loading releases…</p></div>
<noscript><p>JavaScript is needed to load live releases. Browse them on <a href="${REPO_URL}/releases">GitHub Releases</a>.</p></noscript>
<p class="muted small">Rate-limited or offline? Open <a class="ext" href="${REPO_URL}/releases" rel="noopener">GitHub Releases</a> directly.</p>
</section>${dataScript(env)}`;
}

// ---------------------------------------------------------------------------
// Home
// ---------------------------------------------------------------------------

const COPY_ICON =
  '<svg class="i-copy" width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" aria-hidden="true"><rect x="5.500" y="5.500" width="8" height="8" rx="1.500"/><path d="M10.500 3.500v-.5a1 1 0 0 0-1-1H3.500a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h.5"/></svg>';

function homeBody(env) {
  const { base } = env;
  const publishable = libbhooPublishable();
  const libVersion = readVersion(COMPONENTS[0]) || '0.1.0';
  const minor = libVersion.split('.').slice(0, 2).join('.');
  const launcher = readRepoFile('codes/libs/ui/libbhoo/examples/launcher.rs').trim();
  const badges = ['GLES 3.0', 'Wayland', 'WASM', 'Material 3', 'Android XML', 'GPL-3.0-only']
    .map((b) => `<li class="badge">${b}</li>`)
    .join('');
  const grid = COMPONENTS.map((c) => {
    const v = readVersion(c);
    return `<article class="card comp-card" data-component="${c.id}">
  <div class="comp-top"><span class="kind kind-${c.kind}">${KIND_LABEL[c.kind]}</span><span class="ver" title="Version in source">${v ? `v${v}` : ''}</span></div>
  <h3><a href="${base}${c.doc}">${escapeHtml(c.name)}</a></h3>
  <p>${escapeHtml(c.summary)}</p>
  <div class="live-slot" data-live-for="${c.id}"><span class="muted small">Checking GitHub releases…</span></div>
</article>`;
  }).join('\n');

  const unpublishedNote = publishable
    ? ''
    : `<p class="hero-note">libbhoo is not on crates.io yet. Until it is, depend on it from Git (see <a href="#libbhoo">the quickstart</a>).</p>`;
  const cargoSnippet = publishable
    ? `[dependencies]\nlibbhoo = { version = "${minor}", features = ["wayland"] }`
    : `[dependencies]\nlibbhoo = { git = "${REPO_URL}", features = ["wayland"] }`;

  return `
<section class="hero">
  <div class="hero-bg" aria-hidden="true"><div class="hero-glow"></div><div class="hero-grid-lines"></div>${logoSvg({ size: 520 })}</div>
  <div class="wrap hero-inner">
    <div class="hero-copy">
      <p class="eyebrow"><span class="dot"></span>Open-source mobile Linux</p>
      <h1>A mobile Linux stack, <span class="grad">from compositor to keyboard.</span></h1>
      <p class="lede">SpectreOS is an open-source mobile Linux operating system, Wayland compositor/shell, UI toolkit, native application suite and Android compatibility stack, organised as one monorepo around a Rust toolkit called <a href="${base}docs/libbhoo/">libbhoo</a>.</p>
      <div class="cta-row">
        <a class="btn btn-primary" href="${base}docs/getting-started/">Get started</a>
        <a class="btn" href="${base}docs/releases/#live-releases">Download releases</a>
        <button type="button" class="chip copy-chip" data-copy="cargo add libbhoo" aria-label="Copy: cargo add libbhoo"><span class="prompt">$</span><code>cargo add libbhoo</code>${COPY_ICON}</button>
      </div>
      ${unpublishedNote}
      <ul class="badge-row" aria-label="Highlights">${badges}</ul>
    </div>
    <div class="term" role="img" aria-label="Terminal showing how to clone SpectreOS and run the Settings app preview">
      <div class="term-bar"><span></span><span></span><span></span><b>~/spectreos</b></div>
<pre><code><span class="tok-c"># clone, then use the unified app commands</span>
<span class="tok-p">$</span> git clone --recurse-submodules \\
    ${REPO_URL}.git
<span class="tok-p">$</span> cd spectreos
<span class="tok-p">$</span> cargo app-check all
<span class="tok-p">$</span> cargo app-run settings   <span class="tok-c"># Wayland preview window</span>
<span class="tok-p">$</span> make -C codes/libs/ui/libbhoo run-launcher</code></pre>
    </div>
  </div>
</section>

<section class="section wrap" id="stack">
  <div class="section-head"><p class="eyebrow">Architecture</p><h2>One stack, four layers</h2>
  <p class="lede">From the phone's operating system up to the apps in your hand. Wisp and every native app are drawn by libbhoo.</p></div>
  <div class="stack">
    <ol class="layers">
      <li class="layer"><span class="layer-n">01</span><div><h3><a href="${base}docs/spectref1/">Spectre NixOS · Poco F1</a></h3><p>Mobile NixOS port for the Xiaomi Pocophone F1 (<code>beryllium</code>): mainline SDM845 kernel with device patches, firmware verification and flashing scripts. <em>Experimental.</em></p><p class="tags"><span>Mobile NixOS</span><span>SDM845</span><span>aarch64</span></p></div></li>
      <li class="layer-arrow" aria-hidden="true"></li>
      <li class="layer"><span class="layer-n">02</span><div><h3><a href="${base}docs/wisp/">Wisp</a></h3><p>Unified Wayland compositor on an embedded Smithay: DRM/KMS, GBM, EGL/GLES3 and libinput. System UI, lock screen, launcher, multitasking and the notification service.</p><p class="tags"><span>Smithay</span><span>DRM/KMS</span><span>Freedesktop</span></p></div></li>
      <li class="layer-arrow" aria-hidden="true"></li>
      <li class="layer"><span class="layer-n">03</span><div><h3><a href="${base}docs/libbhoo/">libbhoo</a></h3><p>Immediate-mode-inspired Rust UI toolkit with analytical motion, Material Design 3 components, an Android XML runtime, and GLES 3.0, Wayland and WebAssembly backends.</p><p class="tags"><span>Rust</span><span>GLES 3.0</span><span>Material 3</span></p></div></li>
      <li class="layer-arrow" aria-hidden="true"></li>
      <li class="layer"><span class="layer-n">04</span><div><h3><a href="${base}docs/apps/">Native apps</a></h3><p>Rust programs written against libbhoo.</p><p class="tags"><a href="${base}docs/apps/#keyboard">Keyboard</a><a href="${base}docs/apps/#settings">Settings</a><a href="${base}docs/apps/#files">Files</a><a href="${base}docs/apps/#calendar">Calendar</a></p></div></li>
    </ol>
    <aside class="side-module">
      <span class="layer-n">+</span><h3><a class="ext" href="${SHIRAZ_URL}" rel="noopener">Shiraz</a></h3>
      <p>Android compatibility layer (ART, SDL2, EGL/GLES, Skia Ganesh and OS bridges) that runs Android APKs natively on Linux. Integrated into Wisp and Spectre NixOS.</p>
      <p class="muted small">Lives in its own repository, mounted at <code>codes/shiraz-core</code> as a submodule.</p>
      <a class="btn btn-sm" href="${SHIRAZ_URL}" rel="noopener">spectre-oss/shiraz →</a>
    </aside>
  </div>
</section>

<section class="section wrap" id="components">
  <div class="section-head"><p class="eyebrow">Components &amp; releases</p><h2>Versioned independently</h2>
  <p class="lede">Each component has its own version and release tag. The badge shows the version in source today; the line beneath is the latest GitHub release, fetched live.</p></div>
  <div class="grid comp-grid">${grid}</div>
  <p class="muted small center-note">Release downloads come from <a class="ext" href="${REPO_URL}/releases" rel="noopener">GitHub Releases</a>. <a href="${base}docs/releases/">How releases work →</a></p>
</section>
${dataScript(env)}

<section class="section wrap" id="libbhoo">
  <div class="section-head"><p class="eyebrow">libbhoo</p><h2>A small UI toolkit with a real window</h2>
  <p class="lede">The Cargo package is <code>libbhoo</code>; the Rust API is imported as <code>bhoo</code>. This is the repository's own launcher example, verbatim.</p></div>
  <div class="split">
    <div>
      <h3 class="plain">Add it to a crate</h3>
      ${codeBlockHtml(cargoSnippet, 'toml')}
      <p class="small muted">${publishable ? 'Or run <code>cargo add libbhoo --features wayland</code>.' : 'Once published, <code>cargo add libbhoo --features wayland</code> will work too.'} The <code>wayland</code> feature adds the native window and the OpenGL ES 3.0 renderer.</p>
      <h3 class="plain">Run the example</h3>
      ${codeBlockHtml('cd codes/libs/ui/libbhoo\nmake run-launcher\n# or, directly:\ncargo run --release --features wayland --example launcher', 'sh')}
      <ul class="badge-row">${badges}</ul>
    </div>
    <div>
      <h3 class="plain"><code>examples/launcher.rs</code></h3>
      ${codeBlockHtml(launcher, 'rust')}
      <p class="small muted">Requires Linux, a Wayland compositor, EGL with OpenGL ES 3.0+, libxkbcommon and a TTF/OTF font. <a href="${base}docs/libbhoo/">Read the full guide →</a></p>
    </div>
  </div>
</section>

<section class="section wrap" id="apps">
  <div class="section-head"><p class="eyebrow">Native apps</p><h2>Built on the same toolkit</h2></div>
  <div class="grid apps-grid">
    <article class="card"><h3><a href="${base}docs/apps/#keyboard">Keyboard</a></h3><p>Wayland on-screen keyboard with English, Persian (ZWNJ half-space), Terminal and Emoji layouts, controlled over D-Bus as <code>org.spectreos.OSK0</code>.</p></article>
    <article class="card"><h3><a href="${base}docs/apps/#settings">Settings</a></h3><p>Android 13 / AOSP-style System Settings, kept in sync with <code>~/.config/wisp/</code> so the standalone app and Wisp agree.</p></article>
    <article class="card"><h3><a href="${base}docs/apps/#files">Files</a></h3><p>File manager and XDG file chooser with Trash, recursive search, image previews and ZIP support.</p></article>
    <article class="card"><h3><a href="${base}docs/apps/#calendar">Calendar</a></h3><p>Day, Week, Month and Agenda views, recurring events, reminders and iCalendar import/export, embedded in Wisp.</p></article>
    <article class="card"><h3><a href="${base}docs/apps/#bhoo-web-demos">Bhoo web demos</a></h3><p>WebAssembly playground and a side-by-side comparison with Angular Material, served locally.</p></article>
    <article class="card card-cta"><h3><a href="${base}docs/writing-apps/">Write your own</a></h3><p>Native apps are Rust plus libbhoo. Learn the layout, Cargo commands, browser debugger and latency rules.</p></article>
  </div>
</section>

<section class="wrap cta-band">
  <div><h2>Read the docs, build the stack.</h2><p>Everything here is generated from the documentation in the repository.</p></div>
  <div class="cta-row"><a class="btn btn-primary" href="${base}docs/">Browse documentation</a><a class="btn" href="${REPO_URL}" rel="noopener">View on GitHub</a></div>
</section>`;
}

// ---------------------------------------------------------------------------
// Page list
// ---------------------------------------------------------------------------

export function buildPages(env) {
  const pages = [];
  const add = (page, build) => pages.push({ layout: 'docs', ...page, build });

  add(
    { id: 'home', home: true, url: '', title: 'SpectreOS', layout: 'home', section: 'home',
      description: 'SpectreOS is an open-source mobile Linux operating system, Wayland compositor/shell, Rust UI toolkit and native app suite.' },
    (pb) => pb.html(homeBody(env)),
  );

  add(
    { id: 'docs', url: 'docs/', title: 'Documentation', section: 'docs', edit: 'README.md',
      description: 'Overview of the SpectreOS monorepo: components, repository layout and where to read next.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Documentation', title: 'SpectreOS documentation' }));
      pb.md(content('docs-index-lede.md'));
      pb.html(`<div class="grid doc-cards">${[
        ['getting-started', 'Getting started', 'Prerequisites, cloning with submodules and the unified cargo app-* commands.'],
        ['libbhoo', 'libbhoo', 'The Rust UI toolkit: features, backends, frame model and validation.'],
        ['wisp', 'Wisp', 'Wayland compositor, system UI, lock screen and multitasking.'],
        ['apps', 'Native apps', 'Keyboard, Settings, Files, Calendar and the web demos.'],
        ['writing-apps', 'Writing apps', 'App structure, Cargo commands, web debugging and responsiveness.'],
        ['spectref1', 'Spectre NixOS · Poco F1', 'Mobile NixOS port, kernel, firmware and flashing guide.'],
        ['releases', 'Releases & versioning', 'Independent versions, release tags and live downloads.'],
        ['security', 'Security & licensing', 'Reporting vulnerabilities and third-party licenses.'],
      ].map(([id, t, d]) => {
        const it = flatNav().find((n) => n.id === id);
        return `<a class="card doc-card" href="${env.base}${it.url}"><h3>${escapeHtml(t)}</h3><p>${escapeHtml(d)}</p></a>`;
      }).join('')}</div>`);
      pb.file('README.md', { section: 'Repository Structure', includeHeading: true });
      pb.file('README.md', { section: 'Documentation Index', includeHeading: true });
    },
  );

  add(
    { id: 'getting-started', url: 'docs/getting-started/', title: 'Getting started', section: 'docs', edit: 'README.md',
      description: 'Prerequisites, cloning SpectreOS with submodules, and the unified cargo app-* build and test commands.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Start', title: 'Getting started', lede: 'Clone the monorepo, install the toolchain and use the shared task runner to build, test and preview apps.' }));
      pb.md('Everything is built from a single clone. The commands below come straight from the repository README; once the toolchain is installed, `cargo app-help` prints the full usage of the task runner.');
      pb.file('README.md', { section: 'Getting Started', offset: -1 });
      pb.file('README.md', {
        section: 'Common Workflows', offset: -1,
        exclude: ['Shiraz Android Compatibility Layer (`codes/shiraz-core`)'],
      });
      pb.md(`## Shiraz\n\nShiraz, the Android compatibility layer, has its own build and documentation in its [separate repository](${SHIRAZ_URL}). It is checked out at \`codes/shiraz-core\` when you clone with submodules.\n\n## Next steps\n\n- Learn the toolkit: [libbhoo](site:docs/libbhoo/)\n- Create an app: [Writing SpectreOS apps](site:docs/writing-apps/)\n- See how components are versioned: [Releases](site:docs/releases/)`);
    },
  );

  add(
    { id: 'libbhoo', url: 'docs/libbhoo/', title: 'libbhoo', section: 'libbhoo', edit: 'codes/libs/ui/libbhoo/README.md',
      description: 'libbhoo: the Rust UI toolkit behind Wisp and the SpectreOS apps. Install, features, backends, frame model and validation.' },
    (pb) => {
      const publishable = libbhooPublishable();
      const minor = (readVersion(COMPONENTS[0]) || '0.1.0').split('.').slice(0, 2).join('.');
      pb.html(head({ eyebrow: 'Component · crate', title: 'libbhoo', lede: 'Minimal GUI foundations for SpectreOS, written in Rust. Package libbhoo, Rust API bhoo.' }));
      pb.md(
        'Inspired by CUI\'s immediate-mode approach, the core provides controlled widgets, shared pointer/keyboard state, focus, bounded motion and effects, cached labels, semantic snapshots, clipping, fixed-height list virtualization, and Android XML layout/widget support. The Cargo package is **`libbhoo`** and the library is imported as **`bhoo`**; every native SpectreOS app is written in Rust against it.\n\n' +
          '## Install\n\n' +
          (publishable
            ? '```sh\ncargo add libbhoo --features wayland\n```\n\nor in `Cargo.toml`:\n\n'
            : '> [!NOTE]\n> `libbhoo` is not published to crates.io yet (`publish = false` in its `Cargo.toml` when this site was built). Until it is, depend on it from Git. The crate is found by name inside the monorepo.\n\n') +
          '```toml\n[dependencies]\n' +
          (publishable ? `libbhoo = { version = "${minor}", features = ["wayland"] }` : `libbhoo = { git = "${REPO_URL}", features = ["wayland"] }`) +
          '\n```\n\n```rust\nuse bhoo::window::{self, FrameAction, WindowOptions};\n```\n\n' +
          '## Feature flags\n\n| Feature | What it enables |\n| --- | --- |\n| *(default)* | The core engine only: `Ui`, `Frame`, input, motion, widgets and the Android XML runtime, with no GPU or windowing dependencies. |\n| `gles` | The OpenGL ES building blocks: `glow`, `harfrust`, `skrifa`, `zeno`, `unicode-bidi` and `bytemuck` for text shaping, glyph rasterization and vertex upload. |\n| `wayland` | Implies `gles`, and adds `winit`, `glutin`, `glutin-winit` and `raw-window-handle` for a native Wayland window with an explicitly requested GLES 3.0 context. |\n',
        { from: '' },
      );
      pb.md(
        '## Architecture and backends\n\n' +
          '- **Native Wayland + GLES 3.0.** `winit` provides the Wayland window and events, `glutin` the EGL surface with VSync, `glow` the GLES calls with custom GLSL ES 3.00 shaders (`src/platform/gles/shaders.rs`). Text and geometry share one indexed draw batch with premultiplied alpha. It requires Linux, a Wayland compositor, EGL with OpenGL ES 3.0 or newer, libxkbcommon and a TTF/OTF font, and does not fall back to X11 or desktop OpenGL.\n' +
          '- **WebAssembly.** The same Rust core compiles to `wasm32-unknown-unknown`. The web demos paint the toolkit\'s `DrawCommand`s through `codes/apps/bhoo-web/site/canvas-painter.js` (see [Bhoo web demos](site:docs/apps/#bhoo-web-demos)).\n' +
          '- **Design systems.** `src/systems/material/` holds Material Design 3 native widgets (one file per family) and `src/systems/android/` holds Android state models and the XML layout/menu runtime. Motion is analytical and critically damped; shared physics (`motion.rs`, `timing.rs`, `scroll.rs`, `ripple.rs`) stays design-system-agnostic.\n' +
          '- **Headless tests.** A Weston + Mesa software-rendering test checks launcher and gallery pixels at 1× and 2× scale.\n\n' +
          '## Immediate-mode frame model\n\n' +
          'You hand `window::run` a callback that is invoked once per frame. Inside it you describe the UI with widgets, and the callback returns a `FrameAction`:\n\n' +
          '- `FrameAction::Wait` is the normal return. The runner still requests more frames while core effects are active, and otherwise waits for input or tooltip deadlines.\n' +
          '- `FrameAction::Redraw` asks for another frame for application-owned animation.\n' +
          '- `FrameAction::Exit` submits the final frame and closes.\n\n' +
          'CPU and GPU buffers retain capacity, labels use a bounded shared-text cache, and the glyph atlas is uploaded only when it changes.\n\n' +
          'This is the repository\'s launcher example (`examples/launcher.rs`):\n\n' +
          '```rust\n' + readRepoFile('codes/libs/ui/libbhoo/examples/launcher.rs').trim() + '\n```\n\n' +
          '## Run the examples\n\nFrom `codes/libs/ui/libbhoo/` (run `make help` for every build, test, web and WebXDC target):\n\n```sh\nmake help\nmake test\nmake run-launcher\nmake run-components\nmake webxdc-gallery\n```\n\nThe launcher opens a window with a working click counter and a close button (Escape also closes it) and must be run inside an active Wayland session. If no default font can be found, supply one explicitly, which is useful on NixOS:\n\n```sh\nBHOO_FONT=/absolute/path/to/font.ttf cargo run --release --features wayland --example launcher\n```\n',
      );
      pb.file('codes/libs/ui/libbhoo/README.md', { section: 'Project Layout & Development Guide', includeHeading: true });
      pb.file('codes/libs/ui/libbhoo/README.md', { section: 'Validation', includeHeading: true });
      pb.md(
        '## More documentation\n\nTechnical design documents, the component catalog, motion records and the Android UI and Material Design 3 parity goal live in the repository under [`codes/libs/ui/libbhoo/docs/`](repo:codes/libs/ui/libbhoo/docs/README.md). Components are documented one family per file, with their variants, tokens and motion.\n',
      );
      pb.file('codes/libs/ui/libbhoo/DEVELOPMENT.md', {
        includeHeading: true, offset: 1,
        idForHeading: (_t, level) => (level === 1 ? 'development-guide' : undefined),
      });
    },
  );

  add(
    { id: 'wisp', url: 'docs/wisp/', title: 'Wisp', section: 'wisp', edit: 'codes/apps/wisp/README.md',
      description: 'Wisp: the SpectreOS Wayland compositor, system UI, lock screen and launcher, built on Smithay and libbhoo.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Component · app', title: 'Wisp', lede: 'The unified Wayland compositor, System UI, Lock Screen and Launcher for SpectreOS.' }));
      pb.md('Wisp runs natively on device (`--features compositor`), in a desktop preview window (`--features wayland`), or embedded in WebAssembly. It is released as a binary named `wisp-linux-x86_64`; see [Releases](site:docs/releases/).');
      pb.file('codes/apps/wisp/README.md', { stripTitle: true });
      pb.md('## More documentation\n\nArchitecture and design notes are in [`codes/apps/wisp/docs/`](repo:codes/apps/wisp/docs/README.md). Wisp embeds [Smithay](repo:codes/apps/wisp/smithay/LICENSE.txt) under the MIT license; Android app support comes from [Shiraz](' + SHIRAZ_URL + ').');
    },
  );

  add(
    { id: 'apps', url: 'docs/apps/', title: 'Native apps', section: 'apps', edit: 'codes/apps/README.md',
      description: 'SpectreOS native apps: Keyboard, Settings, Files, Calendar and the Bhoo web demos.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Components', title: 'Native apps', lede: 'Applications and interactive demos built on libbhoo.' }));
      pb.file('codes/apps/README.md', { stripTitle: true });
      const section = (rel, id, extra = {}) => pb.file(rel, { offset: 1, includeHeading: true, idForHeading: (t, l) => (l === 1 ? id : `${id.replace(/-demos$/, '')}-${slugify(t)}`), ...extra });
      section('codes/apps/keyboard/README.md', 'keyboard');
      section('codes/apps/settings/README.md', 'settings');
      section('codes/apps/files/README.md', 'files');
      section('codes/apps/calendar/README.md', 'calendar');
      section('codes/apps/bhoo-web/README.md', 'bhoo-web-demos');
    },
  );

  const writing = [
    ['writing-apps', 'docs/writing-apps/', 'Writing SpectreOS apps', 'README.md', 'Native SpectreOS apps are written in Rust and use libbhoo for their UI.'],
    ['app-structure', 'docs/writing-apps/app-structure/', 'App structure, assets and tests', 'app-structure.md', 'The required codes/apps/<app>/ layout, Desktop Entry files and test organization.'],
    ['cargo-commands', 'docs/writing-apps/cargo-commands/', 'Cargo commands', 'cargo-commands.md', 'cargo dev, release, app-test, app-clean and test-web workflows.'],
    ['web-debugging', 'docs/writing-apps/web-debugging/', 'Web debugging', 'web-debugging.md', 'The debug-only WASM browser workbench and reusable test-case buttons.'],
    ['responsiveness', 'docs/writing-apps/responsiveness/', 'Responsiveness', 'responsiveness.md', 'Drawing the first frame, background work, caching and measuring launch latency.'],
  ];
  for (const [id, url, title, file, desc] of writing) {
    const rel = `docs/writing-spectreos-apps/${file}`;
    add({ id, url, title, section: 'docs', edit: rel, description: desc }, (pb) => {
      pb.html(head({ eyebrow: id === 'writing-apps' ? 'Build apps' : 'Writing apps', title }));
      pb.file(rel, { stripTitle: true });
    });
  }

  add(
    { id: 'spectref1', url: 'docs/spectref1/', title: 'Spectre NixOS · Poco F1', section: 'docs', edit: 'codes/os/specrte-nix-f1/README.md',
      description: 'Experimental Mobile NixOS port for the Xiaomi Pocophone F1 (beryllium): kernel, firmware, build and flashing safety.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Component · OS', title: 'Spectre NixOS · Poco F1', lede: 'Experimental Mobile NixOS port for the Xiaomi Pocophone F1 (beryllium, SDM845).' }));
      pb.md(content('spectref1-lede.md'), { from: 'codes/os/specrte-nix-f1/README.md' });
      pb.file('codes/os/specrte-nix-f1/README.md', { stripTitle: true, offset: 1 });
    },
  );

  add(
    { id: 'releases', url: 'docs/releases/', title: 'Releases & versioning', section: 'releases', edit: 'landing/src/content/releases.md',
      description: 'How SpectreOS components are versioned and released independently, plus a live GitHub Releases explorer.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Project', title: 'Releases & versioning', lede: 'Independent per-component versions, binary-only app releases and live downloads.' }));
      pb.md(content('releases.md'));
      pb.html(componentsTable(env));
      pb.md('## Live releases');
      pb.html(liveReleasesWidget(env));
    },
  );

  add(
    { id: 'security', url: 'docs/security/', title: 'Security & licensing', section: 'docs', edit: 'SECURITY.md',
      description: 'How to report vulnerabilities privately, and the licenses covering SpectreOS and its third-party components.' },
    (pb) => {
      pb.html(head({ eyebrow: 'Project', title: 'Security & licensing' }));
      pb.file('SECURITY.md', { stripTitle: true });
      pb.file('README.md', { section: 'Security & Licensing', includeHeading: true });
    },
  );

  return pages;
}

export { COPY_ICON };
