// HTML templates for the SpectreOS site: document shell, header, sidebar, footer.
import { escapeHtml } from './markdown.mjs';
import { REPO_URL, SITE_ORIGIN, SHIRAZ_URL, REPO_BRANCH } from './repo.mjs';

export function logoSvg({ size = 28, title = '' } = {}) {
  return (
    `<svg class="logo" width="${size}" height="${size}" viewBox="0 0 32 32" fill="none" aria-hidden="${title ? 'false' : 'true'}"${title ? ` role="img"` : ''}>` +
    (title ? `<title>${escapeHtml(title)}</title>` : '') +
    `<path d="M16 3.5c-6.3 0-10.5 4.7-10.5 10.6V27c0 .9 1 1.3 1.6.7l2.9-2.9a1 1 0 0 1 1.4 0l2.9 2.9a1.4 1.4 0 0 0 2 0l2.9-2.9a1 1 0 0 1 1.4 0l2.9 2.9c.6.6 1.6.2 1.6-.7V14.100C26.500 8.200 22.300 3.500 16 3.500Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>` +
    `<circle class="logo-eye" cx="12" cy="14.500" r="1.900" fill="var(--accent)"/>` +
    `<circle class="logo-eye" cx="20" cy="14.500" r="1.900" fill="var(--accent)"/>` +
    `</svg>`
  );
}

export const faviconSvg = (dark = true) =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="none">` +
  `<rect width="32" height="32" rx="7" fill="#07090d"/>` +
  `<path d="M16 5.500c-5.500 0-9 4.100-9 9.200V26c0 .8.900 1.100 1.400.6l2.500-2.500a.9.9 0 0 1 1.200 0l2.500 2.500a1.200 1.200 0 0 0 1.800 0l2.500-2.500a.9.9 0 0 1 1.200 0l2.500 2.500c.5.500 1.400.2 1.400-.6V14.700c0-5.100-3.500-9.200-9-9.200Z" stroke="#e6edf5" stroke-width="1.800" stroke-linejoin="round"/>` +
  `<circle cx="12.500" cy="15" r="1.700" fill="#7fe3ee"/><circle cx="19.500" cy="15" r="1.700" fill="#7fe3ee"/></svg>`;

const ICON = {
  github:
    '<svg width="18" height="18" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>',
  search:
    '<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.600" stroke-linecap="round" aria-hidden="true"><circle cx="7" cy="7" r="4.800"/><path d="m10.600 10.600 3.400 3.400"/></svg>',
  menu:
    '<svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.700" stroke-linecap="round" aria-hidden="true"><path d="M3 6h14M3 10h14M3 14h14"/></svg>',
  sun:
    '<svg class="i-sun" width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.600" stroke-linecap="round" aria-hidden="true"><circle cx="10" cy="10" r="3.500"/><path d="M10 2v2M10 16v2M2 10h2M16 10h2M4.300 4.300l1.400 1.400M14.300 14.300l1.400 1.400M4.300 15.700l1.400-1.400M14.300 5.700l1.400-1.400"/></svg>',
  moon:
    '<svg class="i-moon" width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.600" stroke-linejoin="round" aria-hidden="true"><path d="M16.500 12A7 7 0 0 1 8 3.500 7 7 0 1 0 16.500 12Z"/></svg>',
};
export { ICON };

const THEME_BOOT = `(function(){try{var t=localStorage.getItem('spectreos-theme');document.documentElement.dataset.theme=(t==='light'||t==='dark')?t:'dark';}catch(e){document.documentElement.dataset.theme='dark';}})();`;

function topNav(base, current) {
  const link = (id, label, url) =>
    `<a href="${base}${url}"${current === id ? ' aria-current="page"' : ''}>${label}</a>`;
  return [
    link('docs', 'Docs', 'docs/'),
    link('libbhoo', 'libbhoo', 'docs/libbhoo/'),
    link('wisp', 'Wisp', 'docs/wisp/'),
    link('apps', 'Apps', 'docs/apps/'),
    link('releases', 'Releases', 'docs/releases/'),
  ].join('');
}

function header({ base, section }) {
  return `<header class="site-header">
  <div class="header-inner">
    <button type="button" class="icon-btn menu-btn" id="menu-btn" aria-label="Toggle navigation" aria-controls="sidebar" aria-expanded="false">${ICON.menu}</button>
    <a class="brand" href="${base}" aria-label="SpectreOS home">${logoSvg({ size: 26 })}<span class="brand-name">Spectre<b>OS</b></span></a>
    <nav class="top-nav" aria-label="Primary">${topNav(base, section)}</nav>
    <div class="header-actions">
      <button type="button" class="search-btn" id="search-btn" aria-label="Search documentation" aria-keyshortcuts="/ Control+K">${ICON.search}<span class="search-label">Search</span><kbd>/</kbd></button>
      <button type="button" class="icon-btn" id="theme-btn" aria-label="Toggle light/dark theme">${ICON.sun}${ICON.moon}</button>
      <a class="icon-btn" href="${REPO_URL}" aria-label="SpectreOS on GitHub" rel="noopener">${ICON.github}</a>
    </div>
  </div>
</header>`;
}

function sidebar({ base, nav, currentId }) {
  const groups = nav
    .map((g) => {
      const items = g.items
        .map((it) => {
          const active = it.id === currentId;
          const child = (it.children || [])
            .map(
              (c) =>
                `<li><a class="nav-child" href="${base}${c.url}"${c.id === currentId ? ' aria-current="page"' : ''}>${escapeHtml(c.title)}</a></li>`,
            )
            .join('');
          return `<li><a href="${base}${it.url}"${active ? ' aria-current="page"' : ''}>${escapeHtml(it.title)}</a>${child ? `<ul>${child}</ul>` : ''}</li>`;
        })
        .join('');
      return `<div class="nav-group"><p class="nav-title">${escapeHtml(g.title)}</p><ul>${items}</ul></div>`;
    })
    .join('');
  return `<aside class="sidebar" id="sidebar" aria-label="Documentation"><nav>${groups}</nav>
  <div class="nav-group nav-external"><p class="nav-title">Elsewhere</p><ul>
    <li><a class="ext" href="${REPO_URL}" rel="noopener">SpectreOS on GitHub</a></li>
    <li><a class="ext" href="${SHIRAZ_URL}" rel="noopener">Shiraz (separate repo)</a></li>
  </ul></div></aside><div class="scrim" id="scrim" hidden></div>`;
}

function tocHtml(headings) {
  const items = headings.filter((h) => h.level === 2 || h.level === 3);
  if (items.length < 2) return '<aside class="toc" aria-hidden="true"></aside>';
  return `<aside class="toc" aria-label="On this page"><p class="toc-title">On this page</p><ul>${items
    .map((h) => `<li class="toc-l${h.level}"><a href="#${escapeHtml(h.id)}">${escapeHtml(h.text)}</a></li>`)
    .join('')}</ul></aside>`;
}

function pager({ base, prev, next }) {
  if (!prev && !next) return '';
  const a = (p, dir) =>
    p
      ? `<a class="pager-${dir}" href="${base}${p.url}" rel="${dir}"><span>${dir === 'prev' ? 'Previous' : 'Next'}</span><strong>${escapeHtml(p.title)}</strong></a>`
      : '<span></span>';
  return `<nav class="pager" aria-label="Pagination">${a(prev, 'prev')}${a(next, 'next')}</nav>`;
}

function footer({ base, sha }) {
  return `<footer class="site-footer">
  <div class="footer-inner">
    <div class="footer-brand">
      <a class="brand" href="${base}">${logoSvg({ size: 22 })}<span class="brand-name">Spectre<b>OS</b></span></a>
      <p>Open-source mobile Linux: compositor, UI toolkit and native apps in one monorepo. Licensed <a href="${REPO_URL}/blob/${REPO_BRANCH}/LICENSE" rel="noopener">GPL-3.0-only</a>.</p>
    </div>
    <div class="footer-col"><p>Docs</p>
      <a href="${base}docs/getting-started/">Getting started</a>
      <a href="${base}docs/libbhoo/">libbhoo</a>
      <a href="${base}docs/writing-apps/">Writing apps</a>
      <a href="${base}docs/releases/">Releases</a>
    </div>
    <div class="footer-col"><p>Project</p>
      <a href="${REPO_URL}" rel="noopener">GitHub</a>
      <a href="${REPO_URL}/releases" rel="noopener">GitHub Releases</a>
      <a href="${base}docs/security/">Security policy</a>
      <a href="${REPO_URL}/blob/${REPO_BRANCH}/THIRD_PARTY_NOTICES.md" rel="noopener">Third-party notices</a>
      <a href="${SHIRAZ_URL}" rel="noopener">Shiraz</a>
    </div>
  </div>
  <div class="footer-note"><span>© SpectreOS contributors</span>${sha ? `<span>Built from <a href="${REPO_URL}/commit/${sha}" rel="noopener"><code>${sha}</code></a></span>` : ''}</div>
</footer>`;
}

const SEARCH_MODAL = `<div class="search-modal" id="search-modal" hidden>
  <div class="search-backdrop" data-close></div>
  <div class="search-panel" role="dialog" aria-modal="true" aria-label="Search documentation">
    <div class="search-input-row">${ICON.search}<input id="search-input" type="search" placeholder="Search the docs…" autocomplete="off" spellcheck="false" aria-controls="search-results"><kbd>Esc</kbd></div>
    <ul class="search-results" id="search-results" role="listbox"></ul>
    <p class="search-hint"><span><kbd>↑</kbd><kbd>↓</kbd> navigate</span><span><kbd>↵</kbd> open</span><span><kbd>/</kbd> or <kbd>Ctrl</kbd>+<kbd>K</kbd> to search anywhere</span></p>
  </div>
</div>`;

/**
 * Compose a full HTML document.
 * @param {object} o
 */
export function renderDocument(o) {
  const { base, page, body, headings = [], nav, assetVersion = {}, sha = '', prev, next, layout = 'docs' } = o;
  const title = page.home ? 'SpectreOS — open-source mobile Linux' : `${page.title} · SpectreOS`;
  const desc = escapeHtml(page.description || 'SpectreOS: open-source mobile Linux — Wayland compositor, Rust UI toolkit and native apps.');
  const canonical = base === '/' ? '' : `<link rel="canonical" href="${SITE_ORIGIN}${base}${page.url}">`;
  const edit = page.edit
    ? `<a class="edit-link" href="${REPO_URL}/blob/${REPO_BRANCH}/${page.edit}" rel="noopener">View source on GitHub →</a>`
    : '';
  const main =
    layout === 'docs'
      ? `<div class="docs-layout">${sidebar({ base, nav, currentId: page.id })}
  <main id="main" class="docs-main"><article class="prose">${body}</article>${edit ? `<p class="edit-row">${edit}</p>` : ''}${pager({ base, prev, next })}</main>
  ${tocHtml(headings)}</div>`
      : `<main id="main" class="${layout}">${body}</main>`;
  return `<!doctype html>
<html lang="en" data-theme="dark" data-base="${escapeHtml(base)}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escapeHtml(title)}</title>
<meta name="description" content="${desc}">
<meta name="color-scheme" content="dark light">
<meta name="theme-color" content="#07090d">
<meta property="og:site_name" content="SpectreOS">
<meta property="og:title" content="${escapeHtml(title)}">
<meta property="og:description" content="${desc}">
<meta property="og:type" content="website">
${canonical}
${page.noindex ? '<meta name="robots" content="noindex">' : ''}
<link rel="icon" type="image/svg+xml" href="${base}assets/favicon.svg">
<script>${THEME_BOOT}</script>
<link rel="stylesheet" href="${base}assets/style.css?v=${assetVersion.css || '0'}">
<script defer src="${base}assets/app.js?v=${assetVersion.js || '0'}"></script>
</head>
<body class="page-${escapeHtml(page.id)}${layout === 'docs' ? ' has-sidebar' : ''}">
<a class="skip-link" href="#main">Skip to content</a>
${header({ base, section: page.section })}
${main}
${footer({ base, sha })}
${SEARCH_MODAL}
</body>
</html>
`;
}
