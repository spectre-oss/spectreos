// Repository facts: paths, component registry, versions and link resolution.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const here = path.dirname(fileURLToPath(import.meta.url));
export const LANDING_DIR = path.resolve(here, '../..');
export const REPO_ROOT = path.resolve(LANDING_DIR, '..');

export const GITHUB_OWNER = 'spectre-oss';
export const GITHUB_REPO = 'spectreos';
export const REPO_URL = `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}`;
export const SHIRAZ_URL = 'https://github.com/spectre-oss/shiraz';
export const REPO_BRANCH = process.env.REPO_BRANCH || 'main';
export const SITE_ORIGIN = (process.env.SITE_ORIGIN || 'https://spectre-oss.github.io').replace(/\/$/, '');
export const RELEASES_API = `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases`;

export const readRepoFile = (rel) => fs.readFileSync(path.join(REPO_ROOT, rel), 'utf8');
export const repoExists = (rel) => fs.existsSync(path.join(REPO_ROOT, rel));

/**
 * Independently versioned components. Mirrors the release pipeline registry
 * (`scripts/release.py`); `landing` and `shiraz-core` are not versioned here.
 */
export const COMPONENTS = [
  {
    id: 'libbhoo', name: 'libbhoo', kind: 'crate', dir: 'codes/libs/ui/libbhoo', versionFile: '.version',
    summary: 'Rust UI toolkit', doc: 'docs/libbhoo/', artifact: 'crates.io package',
  },
  {
    id: 'wisp', name: 'Wisp', kind: 'app', dir: 'codes/apps/wisp', versionFile: '.version',
    binary: 'wisp', features: 'compositor', summary: 'Wayland compositor & shell', doc: 'docs/wisp/', artifact: 'Linux x86_64 binary',
  },
  {
    id: 'keyboard', name: 'Keyboard', kind: 'app', dir: 'codes/apps/keyboard', versionFile: '.version',
    binary: 'spectre-keyboard', features: 'osk', summary: 'Wayland on-screen keyboard', doc: 'docs/apps/#keyboard', artifact: 'Linux x86_64 binary',
  },
  {
    id: 'settings', name: 'Settings', kind: 'app', dir: 'codes/apps/settings', versionFile: '.version',
    binary: 'spectre-settings', features: 'wayland', summary: 'System Settings', doc: 'docs/apps/#settings', artifact: 'Linux x86_64 binary',
  },
  {
    id: 'files', name: 'Files', kind: 'app', dir: 'codes/apps/files', versionFile: '.version',
    binary: 'spectre-files', features: 'wayland', summary: 'File manager & chooser', doc: 'docs/apps/#files', artifact: 'Linux x86_64 binary',
  },
  {
    id: 'calendar', name: 'Calendar', kind: 'embedded-app', dir: 'codes/apps/calendar', versionFile: '.version',
    summary: 'Calendar (embedded in Wisp)', doc: 'docs/apps/#calendar', artifact: 'release notes only',
  },
  {
    id: 'spectref1', name: 'Spectre NixOS · Poco F1', kind: 'os', dir: 'codes/os/specrte-nix-f1', versionFile: 'VERSION',
    summary: 'Mobile NixOS port for beryllium', doc: 'docs/spectref1/', artifact: 'release notes (images are built locally)',
  },
].map((c) => ({ ...c, tagPrefix: `${c.id}-v` }));

const SEMVER = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;

/** Current in-source version: version file first, then Cargo.toml `[package] version`. */
export function readVersion(c) {
  const vf = path.join(REPO_ROOT, c.dir, c.versionFile);
  if (fs.existsSync(vf)) {
    const v = fs.readFileSync(vf, 'utf8').trim();
    if (SEMVER.test(v)) return v;
  }
  const cargo = path.join(REPO_ROOT, c.dir, 'Cargo.toml');
  if (fs.existsSync(cargo)) {
    const toml = fs.readFileSync(cargo, 'utf8');
    const pkg = toml.split(/^\[/m).find((s) => s.startsWith('package]'));
    const m = pkg && /^version\s*=\s*"([^"]+)"/m.exec(pkg);
    if (m && SEMVER.test(m[1])) return m[1];
  }
  return null;
}

/** Whether libbhoo's Cargo.toml is currently publishable to crates.io. */
export function libbhooPublishable() {
  const cargo = path.join(REPO_ROOT, 'codes/libs/ui/libbhoo/Cargo.toml');
  if (!fs.existsSync(cargo)) return false;
  const pkg = fs.readFileSync(cargo, 'utf8').split(/^\[/m).find((s) => s.startsWith('package]')) || '';
  return !/^publish\s*=\s*false/m.test(pkg);
}

export function gitShortSha() {
  if (process.env.BUILD_SHA) return process.env.BUILD_SHA.slice(0, 7);
  if (process.env.GITHUB_SHA) return process.env.GITHUB_SHA.slice(0, 7);
  try {
    return execFileSync('git', ['rev-parse', '--short=7', 'HEAD'], { cwd: REPO_ROOT, stdio: ['ignore', 'pipe', 'ignore'] })
      .toString()
      .trim();
  } catch {
    return '';
  }
}

/**
 * Build a link resolver for a markdown document.
 *
 * @param {string} srcPath  repo-relative path of the markdown file ('' for authored content)
 * @param {object} env      { base, linkMap: Map<repoPath,string siteUrl>, warnings: string[] }
 */
export function makeLinkResolver(srcPath, env) {
  const { base, linkMap, warnings } = env;
  const site = (p) => base + p.replace(/^\//, '');
  return function resolve(href) {
    if (!href) return href;
    if (href.startsWith('site:')) return site(href.slice(5));
    if (href.startsWith('repo:')) return repoUrl(href.slice(5), srcPath, warnings);
    if (/^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith('//')) return href; // external / mailto
    if (href.startsWith('#')) return href;
    const hashAt = href.indexOf('#');
    const rawPath = hashAt >= 0 ? href.slice(0, hashAt) : href;
    const frag = hashAt >= 0 ? href.slice(hashAt) : '';
    const rel = rawPath.split('?')[0];
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(srcPath || 'x'), rel)).replace(/\/$/, '');
    if (target.startsWith('..')) return repoUrl('', srcPath, warnings);
    const mapped = linkMap.get(target) ?? linkMap.get(`${target}/README.md`);
    if (mapped) {
      const m = /^([^#]*)(#.*)?$/.exec(mapped);
      return site(m[1]) + (frag || m[2] || '');
    }
    return repoUrl(target, srcPath, warnings) + frag;
  };
}

function repoUrl(target, srcPath, warnings) {
  target = target.replace(/^\/+/, '').replace(/\/$/, '');
  if (target === 'codes/shiraz-core') return SHIRAZ_URL;
  if (target.startsWith('codes/shiraz-core/')) return `${SHIRAZ_URL}/blob/HEAD/${target.slice('codes/shiraz-core/'.length)}`;
  if (!target) return REPO_URL;
  const abs = path.join(REPO_ROOT, target);
  if (fs.existsSync(abs)) {
    const kind = fs.statSync(abs).isDirectory() ? 'tree' : 'blob';
    return `${REPO_URL}/${kind}/${REPO_BRANCH}/${target}`;
  }
  warnings?.push(`${srcPath || '(authored)'}: link target not found in repo: ${target}`);
  return `${REPO_URL}/blob/${REPO_BRANCH}/${target}`;
}

export function makeImageResolver(srcPath) {
  return (src) => {
    if (/^(https?:|data:)/i.test(src)) return src;
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(srcPath || 'x'), src));
    return `https://raw.githubusercontent.com/${GITHUB_OWNER}/${GITHUB_REPO}/${REPO_BRANCH}/${target}`;
  };
}
