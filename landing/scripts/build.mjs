#!/usr/bin/env node
// SpectreOS docs & landing site generator. Pure Node.js (>=20), zero dependencies.
//
//   node scripts/build.mjs                  -> landing/dist, BASE_PATH=/
//   BASE_PATH=/spectreos/ node scripts/build.mjs -> subpath build
//   OUT_DIR=/tmp/site node scripts/build.mjs
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { stripTags, escapeHtml } from './lib/markdown.mjs';
import { LANDING_DIR, SITE_ORIGIN, gitShortSha } from './lib/repo.mjs';
import { renderDocument, faviconSvg } from './lib/layout.mjs';
import { buildPages, flatNav, NAV, LINK_MAP, PageBuilder } from './lib/pages.mjs';

export function normalizeBase(b) {
  let base = (b ?? '/').trim() || '/';
  if (!base.startsWith('/')) base = `/${base}`;
  if (!base.endsWith('/')) base += '/';
  return base;
}

const hash = (buf) => crypto.createHash('sha256').update(buf).digest('hex').slice(0, 10);

function searchEntries(page, body, base) {
  const clean = (html) =>
    stripTags(html.replace(/<figcaption>[\s\S]*?<\/figcaption>/g, ' ').replace(/<a class="anchor"[^>]*>#<\/a>/g, '').replace(/<script[\s\S]*?<\/script>/g, ' ').replace(/<\/(p|li|tr|h\d|pre|div|figure)>/g, ' $& '))
      .replace(/\s+/g, ' ')
      .trim();
  const entries = [];
  const re = /<h([23]) id="([^"]+)">([\s\S]*?)<\/h\1>/g;
  const marks = [];
  let m;
  while ((m = re.exec(body))) marks.push({ idx: m.index, end: re.lastIndex, id: m[2], title: clean(m[3]) });
  const intro = clean(body.slice(0, marks[0]?.idx ?? body.length));
  if (intro) entries.push({ t: page.title, u: page.url, p: page.title, x: intro.slice(0, 1200) });
  marks.forEach((mk, i) => {
    const text = clean(body.slice(mk.end, marks[i + 1]?.idx ?? body.length));
    entries.push({ t: mk.title, u: `${page.url}#${mk.id}`, p: page.title, x: text.slice(0, 1200) });
  });
  return entries;
}

export function build({ base: baseIn, outDir: outIn, quiet = false } = {}) {
  const base = normalizeBase(baseIn ?? process.env.BASE_PATH);
  const outDir = path.resolve(outIn ?? process.env.OUT_DIR ?? path.join(LANDING_DIR, 'dist'));
  const t0 = performance.now();
  const warnings = [];
  const env = { base, linkMap: LINK_MAP, warnings };

  fs.rmSync(outDir, { recursive: true, force: true });
  fs.mkdirSync(path.join(outDir, 'assets'), { recursive: true });

  // assets
  const assetsSrc = path.join(LANDING_DIR, 'src/assets');
  const css = fs.readFileSync(path.join(assetsSrc, 'style.css'));
  const js = fs.readFileSync(path.join(assetsSrc, 'app.js'));
  fs.writeFileSync(path.join(outDir, 'assets/style.css'), css);
  fs.writeFileSync(path.join(outDir, 'assets/app.js'), js);
  fs.writeFileSync(path.join(outDir, 'assets/favicon.svg'), faviconSvg());
  const assetVersion = { css: hash(css), js: hash(js) };
  const sha = gitShortSha();

  const pages = buildPages(env);
  const order = flatNav();
  const search = [];
  const written = [];

  const write = (rel, data) => {
    const file = path.join(outDir, rel);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, data);
    written.push(rel);
  };

  for (const page of pages) {
    const pb = new PageBuilder(env);
    page.build(pb);
    const body = pb.toString();
    const idx = order.findIndex((n) => n.id === page.id);
    const html = renderDocument({
      base, page, body, headings: pb.headings, nav: NAV, assetVersion, sha,
      prev: idx > 0 ? order[idx - 1] : null,
      next: idx >= 0 && idx < order.length - 1 ? order[idx + 1] : null,
      layout: page.layout,
    });
    write(`${page.url}index.html`, html);
    if (page.layout === 'docs') search.push(...searchEntries(page, body, base));
    else search.push({ t: 'SpectreOS home', u: '', p: 'Home', x: page.description });
  }

  // 404
  const notFound = {
    id: '404', url: '404.html', title: 'Page not found', section: '', noindex: true, layout: 'home',
    description: 'This page does not exist.',
  };
  write(
    '404.html',
    renderDocument({
      base, page: notFound, assetVersion, sha, nav: NAV, layout: 'home',
      body: `<section class="wrap notfound"><p class="eyebrow">404</p><h1>This page drifted away.</h1><p class="lede">The page you asked for does not exist on this site. Try the search (press <kbd>/</kbd>) or head back.</p><div class="cta-row"><a class="btn btn-primary" href="${base}">Home</a><a class="btn" href="${base}docs/">Documentation</a></div></section>`,
    }),
  );

  write('search-index.json', JSON.stringify(search));
  write('.nojekyll', '');
  write('robots.txt', `User-agent: *\nAllow: /\nSitemap: ${SITE_ORIGIN}${base}sitemap.xml\n`);
  write(
    'sitemap.xml',
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${pages
      .map((p) => `  <url><loc>${escapeHtml(SITE_ORIGIN + base + p.url)}</loc></url>`)
      .join('\n')}\n</urlset>\n`,
  );

  const ms = Math.round(performance.now() - t0);
  if (!quiet) {
    console.log(`SpectreOS site: ${pages.length + 1} pages, ${search.length} search entries -> ${path.relative(process.cwd(), outDir) || '.'} (base ${base}) in ${ms} ms`);
    for (const w of [...new Set(warnings)]) console.warn(`warning: ${w}`);
  }
  return { base, outDir, pages, written, warnings: [...new Set(warnings)], search, ms };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    build();
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
}
