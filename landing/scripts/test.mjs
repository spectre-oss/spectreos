#!/usr/bin/env node
// Automated validation for the SpectreOS site. Run with `npm test` (zero dependencies).
//
// 1. Unit checks for the Markdown compiler and link rewriting.
// 2. Builds the site for BASE_PATH=/ (CLI entry point) and BASE_PATH=/spectreos/ (API).
// 3. Verifies pages, assets, search index, 404, .nojekyll, internal links + anchors,
//    unresolved placeholders, file:// links and the .gitignore rule for landing/dist.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { build } from './build.mjs';
import { renderMarkdown, extractSection, slugify } from './lib/markdown.mjs';
import { makeLinkResolver, LANDING_DIR, REPO_ROOT, REPO_URL, SHIRAZ_URL, SITE_ORIGIN } from './lib/repo.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
let passed = 0;
const failures = [];
function check(cond, msg) {
  if (cond) passed++;
  else {
    failures.push(msg);
    console.error(`  ✗ ${msg}`);
  }
}
const section = (t) => console.log(`\n▸ ${t}`);

const EXPECTED_PAGES = [
  'index.html',
  'docs/index.html',
  'docs/getting-started/index.html',
  'docs/libbhoo/index.html',
  'docs/wisp/index.html',
  'docs/apps/index.html',
  'docs/writing-apps/index.html',
  'docs/writing-apps/app-structure/index.html',
  'docs/writing-apps/cargo-commands/index.html',
  'docs/writing-apps/web-debugging/index.html',
  'docs/writing-apps/responsiveness/index.html',
  'docs/spectref1/index.html',
  'docs/releases/index.html',
  'docs/security/index.html',
  '404.html',
];
const EXPECTED_FILES = [
  ...EXPECTED_PAGES, 'assets/style.css', 'assets/app.js', 'assets/favicon.svg',
  'search-index.json', 'sitemap.xml', 'robots.txt', '.nojekyll',
];

// ---------------------------------------------------------------- unit tests
section('Markdown compiler');
{
  const md = [
    '# Title', '', '## Build & Run', '', 'Some **bold**, *italic*, `a|b` and `x_y_z` text with [a link](docs/foo.md#bar).', '',
    '## Build & Run', '',
    '| Cmd | Use |', '| --- | :-: |', '| `cargo dev [app\\|all]` | build |', '',
    '> [!WARNING]', '> careful', '',
    '- one', '  - nested', '- two', '', '1. first', '2. second', '',
    '```rust', 'fn main() { let s = "hi"; } // <tag>', '```', '', '<b>raw</b> & <app>',
  ].join('\n');
  const headings = [];
  const { html } = renderMarkdown(md, { headings, offset: 1, stripTitle: true, resolveLink: (h) => `R(${h})` });
  check(!/<h1/.test(html) && !/>Title</.test(html), 'stripTitle removes the first H1');
  check(headings.map((h) => h.id).join() === 'build--run,build--run-1', `duplicate heading ids are made unique (${headings.map((h) => h.id)})`);
  check(/<h3 id="build--run">/.test(html), 'heading offset applied');
  check(html.includes('<strong>bold</strong>') && html.includes('<em>italic</em>'), 'bold and italic');
  check(html.includes('<code>a|b</code>') && html.includes('<code>x_y_z</code>'), 'inline code is protected');
  check(html.includes('href="R(docs/foo.md#bar)"'), 'links go through the resolver');
  check(/<table>[\s\S]*<code>cargo dev \[app\|all\]<\/code>/.test(html) && html.includes('text-align:center'), 'tables: escaped pipes and alignment');
  check(html.includes('class="alert alert-warning"'), 'GitHub alerts');
  check(/<ul><li>one<ul><li>nested<\/li><\/ul><\/li><li>two<\/li><\/ul>/.test(html), 'nested tight lists');
  check(html.includes('<ol><li>first</li><li>second</li></ol>'), 'ordered lists');
  check(html.includes('data-lang="rust"') && html.includes('copy-btn') && html.includes('tok-k'), 'fenced code: badge, copy button, highlighting');
  check(html.includes('&lt;tag&gt;') && html.includes('&lt;b&gt;raw&lt;/b&gt;') && html.includes('&lt;app&gt;'), 'raw HTML is escaped');
  check(/<h2 id="setext">/.test(renderMarkdown('Setext\n------\n\ntext\n').html.replace(/<a class="anchor"[^>]*>#<\/a>/, '')), 'setext headings');
  check(slugify('`libbhoo` Dev & Guide') === 'libbhoo-dev--guide', 'GitHub-style slugs');
  const sec = extractSection('# A\n\n## One\n\nx\n\n### Sub\n\ny\n\n## Two\n\nz\n', 'One', { exclude: ['Sub'] });
  check(sec.includes('x') && !sec.includes('y') && !sec.includes('z'), 'extractSection with exclusions');
}

section('Link rewriting');
{
  const env = { base: '/spectreos/', linkMap: new Map([['docs/writing-spectreos-apps/README.md', 'docs/writing-apps/']]), warnings: [] };
  const fromDocs = makeLinkResolver('docs/writing-spectreos-apps/app-structure.md', env);
  check(fromDocs('README.md') === '/spectreos/docs/writing-apps/', 'doc-to-doc link maps to the site page');
  const r = makeLinkResolver('README.md', env);
  check(r('codes/apps/wisp/') === `${REPO_URL}/tree/main/codes/apps/wisp`, 'directory links become GitHub tree URLs');
  check(r('LICENSE') === `${REPO_URL}/blob/main/LICENSE`, 'file links become GitHub blob URLs');
  check(r('codes/shiraz-core') === SHIRAZ_URL, 'shiraz-core links point at the Shiraz repo');
  check(r('https://example.com/x') === 'https://example.com/x' && r('#frag') === '#frag', 'external and fragment links untouched');
  check(r('site:docs/') === '/spectreos/docs/', 'site: scheme honours BASE_PATH');
}

// ------------------------------------------------------------------- builds
function listFiles(dir) {
  const out = [];
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) out.push(...listFiles(p));
    else out.push(p);
  }
  return out;
}

const idCache = new Map();
function idsOf(file) {
  if (!idCache.has(file)) {
    const html = fs.readFileSync(file, 'utf8');
    idCache.set(file, new Set([...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1])));
  }
  return idCache.get(file);
}

function validate(base, outDir, label) {
  section(`Validate ${label} (BASE_PATH=${base})`);
  idCache.clear();
  for (const f of EXPECTED_FILES) {
    const p = path.join(outDir, f);
    const ok = fs.existsSync(p) && (f === '.nojekyll' || fs.statSync(p).size > 0);
    check(ok, `${f} exists${f === '.nojekyll' ? '' : ' and is non-empty'}`);
  }

  // search index
  let index = [];
  try {
    index = JSON.parse(fs.readFileSync(path.join(outDir, 'search-index.json'), 'utf8'));
    check(Array.isArray(index) && index.length > 30, `search-index.json is valid JSON with ${index.length} entries`);
  } catch (e) {
    check(false, `search-index.json is valid JSON (${e.message})`);
  }
  check(index.every((e) => e.t && e.p && typeof e.u === 'string' && typeof e.x === 'string'), 'every search entry has t/u/p/x');
  check(index.some((e) => /cargo app-test/.test(e.x)), 'search index covers command text from the docs');

  const resolveInternal = (url) => {
    // returns { file, hash } or null when not internal
    let u = url;
    if (/^(https?:|mailto:|data:|javascript:|\/\/)/i.test(u)) return null;
    const hashAt = u.indexOf('#');
    const hash = hashAt >= 0 ? decodeURIComponent(u.slice(hashAt + 1)) : '';
    u = hashAt >= 0 ? u.slice(0, hashAt) : u;
    u = u.split('?')[0];
    return { path: u, hash };
  };

  for (const e of index) {
    const r = resolveInternal(e.u);
    const rel = r.path.endsWith('/') || r.path === '' ? `${r.path}index.html` : r.path;
    const f = path.join(outDir, rel);
    const ok = fs.existsSync(f) && (!r.hash || idsOf(f).has(r.hash));
    if (!ok) check(false, `search entry resolves: ${e.u}`);
    else passed++;
  }

  const htmlFiles = listFiles(outDir).filter((f) => f.endsWith('.html'));
  let linkCount = 0;
  for (const file of htmlFiles) {
    const rel = path.relative(outDir, file);
    const html = fs.readFileSync(file, 'utf8');
    const visible = html.replace(/<script[\s\S]*?<\/script>/g, '');

    check(!/\{\{[^}]*\}\}|%%[A-Z_]+%%|\$\{[^}]*\}/.test(visible.replace(/<pre[\s\S]*?<\/pre>/g, '')), `${rel}: no unresolved template placeholders`);
    check(!/\b(undefined|\[object Object\]|NaN)\b/.test(visible.replace(/<pre[\s\S]*?<\/pre>/g, '')), `${rel}: no undefined/NaN leaks`);
    check(!/(?:href|src|action)="file:/i.test(html), `${rel}: no file:// links`);
    check((html.match(/<h1[\s>]/g) || []).length === 1, `${rel}: exactly one <h1>`);
    check(/<title>[^<]+<\/title>/.test(html), `${rel}: has a title`);
    const ids = [...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]);
    check(new Set(ids).size === ids.length, `${rel}: element ids are unique`);
    check(!/<script[^>]+src="https?:/i.test(html), `${rel}: no third-party scripts`);

    for (const m of html.matchAll(/\s(?:href|src)="([^"]*)"/g)) {
      const raw = m[1].replace(/&amp;/g, '&');
      if (raw === '') continue;
      const r = resolveInternal(raw);
      if (!r) continue;
      linkCount++;
      if (r.path === '' && r.hash) {
        check(idsOf(file).has(r.hash), `${rel}: in-page anchor #${r.hash} exists`);
        continue;
      }
      if (!r.path.startsWith('/')) {
        check(false, `${rel}: relative link is not allowed (${raw})`);
        continue;
      }
      if (!r.path.startsWith(base)) {
        check(false, `${rel}: root-relative link ignores BASE_PATH ${base} (${raw})`);
        continue;
      }
      let sub = r.path.slice(base.length);
      if (sub === '' || sub.endsWith('/')) sub += 'index.html';
      const target = path.join(outDir, sub);
      if (!fs.existsSync(target) || !fs.statSync(target).isFile()) {
        check(false, `${rel}: broken internal link ${raw}`);
        continue;
      }
      if (r.hash && target.endsWith('.html') && !idsOf(target).has(r.hash)) {
        check(false, `${rel}: link ${raw} points at a missing anchor`);
        continue;
      }
      passed++;
    }
  }
  console.log(`  checked ${htmlFiles.length} pages and ${linkCount} internal URLs`);

  const home = fs.readFileSync(path.join(outDir, 'index.html'), 'utf8');
  check(home.includes('cargo add libbhoo'), 'home shows the cargo add libbhoo CTA');
  check(home.includes('docs/getting-started/') && home.includes('docs/releases/'), 'home links to Get Started and Releases');
  check(home.includes('data-base="' + base + '"'), 'html data-base matches BASE_PATH');
  for (const b of ['GLES 3.0', 'Wayland', 'WASM', 'Material 3', 'Android XML', 'GPL-3.0-only']) check(home.includes(b), `home has the "${b}" badge`);
  check(home.includes('api.github.com/repos/spectre-oss/spectreos/releases'), 'home embeds the GitHub Releases API endpoint for live fetch');
  const releases = fs.readFileSync(path.join(outDir, 'docs/releases/index.html'), 'utf8');
  check(releases.includes('id="live-releases-widget"') && releases.includes('spectref1-v'), 'releases page has the live explorer and per-component tags');
  const f1Version = fs.readFileSync(path.join(REPO_ROOT, 'codes/os/specrte-nix-f1/VERSION'), 'utf8').trim();
  check(releases.includes(`<code>${f1Version}</code>`), `Spectre NixOS version is read from VERSION (${f1Version})`);
  const asset = fs.readFileSync(path.join(outDir, 'assets/style.css'), 'utf8');
  check(asset.includes('[data-theme="light"]') && asset.includes('[data-theme="dark"]'), 'stylesheet defines dark and light themes');
  const js = fs.readFileSync(path.join(outDir, 'assets/app.js'), 'utf8');
  check(js.includes('spectreos-theme') && js.includes("e.key === '/'") && js.includes('search-index.json'), 'app.js wires theme persistence, "/" search shortcut and the index');
  const nf = fs.readFileSync(path.join(outDir, '404.html'), 'utf8');
  check(nf.includes(`href="${base}"`), '404 links home using BASE_PATH');
  const sitemap = fs.readFileSync(path.join(outDir, 'sitemap.xml'), 'utf8');
  check(sitemap.includes(`${SITE_ORIGIN}${base}docs/libbhoo/`), 'sitemap lists pages under the configured base');
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'spectreos-site-'));
try {
  section('Build via CLI (npm run build equivalent, default base)');
  const defaultOut = path.join(tmp, 'default');
  const env = { ...process.env, OUT_DIR: defaultOut };
  delete env.BASE_PATH;
  const run = spawnSync(process.execPath, [path.join(here, 'build.mjs')], { env, encoding: 'utf8' });
  check(run.status === 0, `node scripts/build.mjs exits 0 ${run.status === 0 ? '' : `\n${run.stderr}`}`);
  if (run.stdout) console.log('  ' + run.stdout.trim().split('\n').join('\n  '));
  check(/base \/\)/.test(run.stdout || ''), 'default BASE_PATH is /');
  validate('/', defaultOut, 'custom-domain root build');

  section('Build with BASE_PATH=/spectreos/');
  const subpathOut = path.join(tmp, 'subpath');
  const res = build({ base: '/spectreos/', outDir: subpathOut, quiet: true });
  check(res.ms < 5000, `subpath build finished in ${res.ms} ms`);
  validate('/spectreos/', subpathOut, 'project-subpath build');

  section('Build normalisation');
  const odd = build({ base: 'docs-site', outDir: path.join(tmp, 'odd'), quiet: true });
  check(odd.base === '/docs-site/', 'BASE_PATH without slashes is normalised');
  validate('/docs-site/', path.join(tmp, 'odd'), 'custom base build');

  section('Repository hygiene');
  const gi = fs.readFileSync(path.join(LANDING_DIR, '.gitignore'), 'utf8').split(/\r?\n/).map((l) => l.trim());
  check(gi.some((l) => l === 'dist' || l === 'dist/' || l === '/dist' || l === '/dist/'), 'landing/.gitignore ignores dist/');
  try {
    execFileSync('git', ['check-ignore', '-q', 'landing/dist/index.html'], { cwd: REPO_ROOT, stdio: 'ignore' });
    check(true, 'git ignores landing/dist/index.html');
  } catch (e) {
    if (e.status === 1) check(false, 'git does not ignore landing/dist/index.html');
    else console.log('  (git unavailable, skipped git check-ignore)');
  }
  try {
    const tracked = execFileSync('git', ['ls-files', 'landing/dist'], { cwd: REPO_ROOT, encoding: 'utf8' }).trim();
    check(tracked === '', 'no built files under landing/dist are tracked');
  } catch { /* not a git checkout */ }
  const pkg = JSON.parse(fs.readFileSync(path.join(LANDING_DIR, 'package.json'), 'utf8'));
  check(pkg.scripts?.build && pkg.scripts?.preview && pkg.scripts?.test, 'package.json defines build, preview and test scripts');
  check(!pkg.dependencies && !pkg.devDependencies, 'package.json has zero dependencies');
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

console.log(`\n${failures.length ? '✗' : '✓'} ${passed} checks passed, ${failures.length} failed`);
if (failures.length) {
  console.error('\nFailures:\n' + failures.map((f) => ` - ${f}`).join('\n'));
  process.exit(1);
}
