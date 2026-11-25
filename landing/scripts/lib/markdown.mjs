// Zero-dependency Markdown -> HTML compiler used by the SpectreOS docs site.
//
// Supported: ATX headings (anchor ids + TOC data), paragraphs, fenced code
// (language badge, copy button, light syntax highlighting), tables with
// alignment, blockquotes + GitHub alerts, nested/ordered/tight/loose lists,
// horizontal rules, inline code, bold/italic/strike, links, images, autolinks
// and backslash escapes. Raw HTML in the source is always escaped.

export function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function decodeEntities(s) {
  return String(s)
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&');
}

export function stripTags(html) {
  return decodeEntities(String(html).replace(/<[^>]*>/g, ''));
}

/** GitHub-style heading slug. */
export function slugify(text) {
  const s = String(text)
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .trim()
    .replace(/\s/g, '-');
  return s || 'section';
}

// ---------------------------------------------------------------------------
// Syntax highlighting (intentionally tiny: comments, strings, keywords, ...)
// ---------------------------------------------------------------------------

const RUST_KW =
  'as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while';

const GRAMMARS = {
  shell: [
    ['c', String.raw`(?<![^\s])#[^\n]*`],
    ['s', String.raw`"(?:\\.|[^"\\])*"|'[^']*'`],
    ['v', String.raw`\$\{?[A-Za-z_]\w*\}?`],
    ['k', String.raw`^[ \t]*[A-Za-z_./~][\w./~-]*`],
    ['f', String.raw`(?<=\s)--?[A-Za-z][\w-]*`],
  ],
  rust: [
    ['c', String.raw`//[^\n]*|/\*[\s\S]*?\*/`],
    ['s', String.raw`"(?:\\.|[^"\\])*"`],
    ['a', String.raw`#!?\[[^\]\n]*\]`],
    ['k', String.raw`\b(?:${RUST_KW.split(' ').join('|')})\b`],
    ['t', String.raw`\b[A-Z][A-Za-z0-9_]*\b`],
    ['n', String.raw`\b\d[\d_.]*\b`],
  ],
  toml: [
    ['c', String.raw`#[^\n]*`],
    ['t', String.raw`^\[[^\]\n]+\]`],
    ['s', String.raw`"(?:\\.|[^"\\])*"|'[^']*'`],
    ['k', String.raw`^[ \t]*[\w.-]+(?=[ \t]*=)`],
    ['n', String.raw`\b(?:true|false|\d[\d_.]*)\b`],
  ],
  json: [
    ['k', String.raw`"(?:\\.|[^"\\])*"(?=\s*:)`],
    ['s', String.raw`"(?:\\.|[^"\\])*"`],
    ['n', String.raw`-?\b\d[\d.eE+-]*\b|\b(?:true|false|null)\b`],
  ],
  python: [
    ['c', String.raw`#[^\n]*`],
    ['s', String.raw`"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'`],
    ['k', String.raw`\b(?:def|class|import|from|return|if|elif|else|for|while|in|not|and|or|with|as|try|except|raise|None|True|False|pass|lambda)\b`],
    ['n', String.raw`\b\d[\d_.]*\b`],
  ],
  ini: [
    ['c', String.raw`^[ \t]*[#;][^\n]*`],
    ['t', String.raw`^\[[^\]\n]+\]`],
    ['k', String.raw`^[ \t]*[\w.@-]+(?=[ \t]*=)`],
  ],
};

const LANG_ALIAS = {
  sh: 'shell', bash: 'shell', shell: 'shell', zsh: 'shell', console: 'shell',
  rust: 'rust', rs: 'rust',
  toml: 'toml', json: 'json', python: 'python', py: 'python',
  ini: 'ini', desktop: 'ini', conf: 'ini',
};

const compiled = new Map();
function grammarFor(lang) {
  const key = LANG_ALIAS[lang];
  if (!key) return null;
  if (!compiled.has(key)) {
    const parts = GRAMMARS[key].map(([name, src]) => `(?<${name}>${src})`);
    compiled.set(key, new RegExp(parts.join('|'), 'gm'));
  }
  return compiled.get(key);
}

export function highlight(code, lang) {
  const re = grammarFor((lang || '').toLowerCase());
  if (!re) return escapeHtml(code);
  re.lastIndex = 0;
  let out = '';
  let last = 0;
  let m;
  while ((m = re.exec(code))) {
    if (m[0] === '') {
      re.lastIndex++;
      continue;
    }
    const cls = Object.keys(m.groups).find((k) => m.groups[k] !== undefined);
    out += escapeHtml(code.slice(last, m.index));
    out += `<span class="tok-${cls}">${escapeHtml(m[0])}</span>`;
    last = m.index + m[0].length;
  }
  return out + escapeHtml(code.slice(last));
}

// ---------------------------------------------------------------------------
// Inline rendering
// ---------------------------------------------------------------------------

const ESCAPABLE = '\\`*_{}[]()#+-.!|<>~';

export function renderInline(text, ctx = {}) {
  const stash = [];
  const hold = (html) => `\u0000${stash.push(html) - 1}\u0000`;
  let s = String(text);

  // 1. code spans
  s = s.replace(/(`+)([\s\S]*?[^`])\1(?!`)/g, (_, _t, body) => {
    let b = body.replace(/\n/g, ' ');
    if (/^ .* $/.test(b) && b.trim() !== '') b = b.slice(1, -1);
    return hold(`<code>${escapeHtml(b)}</code>`);
  });

  // 2. backslash escapes
  s = s.replace(/\\([\\`*_{}\[\]()#+\-.!|<>~])/g, (_, ch) => {
    if (!ESCAPABLE.includes(ch)) return `\\${ch}`;
    return hold(escapeHtml(ch));
  });

  // 3. autolinks <https://...>
  s = s.replace(/<((?:https?:\/\/|mailto:)[^>\s]+)>/g, (_, url) =>
    hold(linkHtml(url, escapeHtml(url), '', ctx)),
  );

  // 4. escape the remainder
  s = escapeHtml(s);

  // 5. images, then links
  s = s.replace(/!\[([^\]]*)\]\(\s*([^()\s]+)(?:\s+&quot;([^&]*)&quot;)?\s*\)/g, (_, alt, src) => {
    const url = ctx.resolveImage ? ctx.resolveImage(decodeEntities(src)) : decodeEntities(src);
    return hold(`<img src="${escapeHtml(url)}" alt="${alt}" loading="lazy">`);
  });
  const linkRe =
    /\[((?:[^\[\]]|\[[^\]]*\])*)\]\(\s*([^()\s]+(?:\([^()\s]*\)[^()\s]*)*)(?:\s+&quot;([^&]*)&quot;)?\s*\)/g;
  s = s.replace(linkRe, (_, label, href, title) =>
    hold(linkHtml(decodeEntities(href), emphasis(label), title || '', ctx)),
  );

  // 6. emphasis
  s = emphasis(s);

  // 7. hard breaks (marker inserted by the paragraph renderer)
  s = s.replace(/\u0001/g, '<br>\n');

  // restore placeholders (may be nested)
  for (let pass = 0; pass < 4 && /\u0000\d+\u0000/.test(s); pass++) {
    s = s.replace(/\u0000(\d+)\u0000/g, (_, n) => stash[Number(n)]);
  }
  return s;
}

function emphasis(s) {
  return s
    .replace(/\*\*(?=\S)([\s\S]*?\S)\*\*/g, '<strong>$1</strong>')
    .replace(/(?<![\w])__(?=\S)([\s\S]*?\S)__(?![\w])/g, '<strong>$1</strong>')
    .replace(/(?<![*\w])\*(?=[^\s*])([^*]*?[^\s*])\*(?!\*)/g, '<em>$1</em>')
    .replace(/(?<![\w])_(?=[^\s_])([^_]*?[^\s_])_(?![\w])/g, '<em>$1</em>')
    .replace(/~~(?=\S)([\s\S]*?\S)~~/g, '<del>$1</del>');
}

function linkHtml(href, labelHtml, title, ctx) {
  const resolved = ctx.resolveLink ? ctx.resolveLink(href) : href;
  const external = /^https?:\/\//i.test(resolved);
  const attrs = [`href="${escapeHtml(resolved)}"`];
  if (title) attrs.push(`title="${title}"`);
  if (external) attrs.push('class="ext"', 'rel="noopener"');
  return `<a ${attrs.join(' ')}>${labelHtml}</a>`;
}

// ---------------------------------------------------------------------------
// Block rendering
// ---------------------------------------------------------------------------

const RE = {
  fence: /^( {0,3})(`{3,}|~{3,})[ \t]*([^`\s]*)[^`]*$/,
  heading: /^ {0,3}(#{1,6})[ \t]+(.+?)(?:[ \t]+#+)?[ \t]*$/,
  hr: /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/,
  quote: /^ {0,3}>/,
  list: /^( {0,3})([-*+]|\d{1,9}[.)])(?:[ \t]+|$)/,
  tableDelim: /^\s*\|?\s*:?-+:?\s*(?:\|\s*:?-+:?\s*)*\|?\s*$/,
  blank: /^\s*$/,
};

const indentOf = (l) => l.match(/^ */)[0].length;

function splitRow(line) {
  let t = line.trim();
  if (t.startsWith('|')) t = t.slice(1);
  if (t.endsWith('|') && !t.endsWith('\\|')) t = t.slice(0, -1);
  const cells = [];
  let cur = '';
  for (let i = 0; i < t.length; i++) {
    const c = t[i];
    if (c === '\\' && t[i + 1] === '|') {
      cur += '|';
      i++;
    } else if (c === '|') {
      cells.push(cur.trim());
      cur = '';
    } else cur += c;
  }
  cells.push(cur.trim());
  return cells;
}

function isTableStart(lines, i) {
  const l = lines[i];
  const n = lines[i + 1];
  if (l === undefined || n === undefined) return false;
  if (!l.includes('|') || !n.includes('-') || !RE.tableDelim.test(n)) return false;
  return splitRow(l).length === splitRow(n).length;
}

function interruptsParagraph(lines, i) {
  const l = lines[i];
  if (RE.fence.test(l) || RE.heading.test(l) || RE.hr.test(l) || RE.quote.test(l)) return true;
  const m = RE.list.exec(l);
  if (m && (/^[-*+]/.test(m[2]) || m[2].startsWith('1'))) return true;
  return /^\s*<!--/.test(l);
}

const ALERT_TITLES = { note: 'Note', tip: 'Tip', important: 'Important', warning: 'Warning', caution: 'Caution' };

export function codeBlockHtml(code, lang) {
  const label = lang && lang !== 'text' ? lang : '';
  const badge = label ? `<span class="code-lang">${escapeHtml(label)}</span>` : '<span class="code-lang"></span>';
  return (
    `<figure class="code"${label ? ` data-lang="${escapeHtml(label)}"` : ''}>` +
    `<figcaption>${badge}<button type="button" class="copy-btn" aria-label="Copy code to clipboard">Copy</button></figcaption>` +
    `<pre><code${label ? ` class="language-${escapeHtml(label)}"` : ''}>${highlight(code, lang)}</code></pre></figure>`
  );
}

function parseBlocks(lines, ctx, tight = false) {
  const out = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (RE.blank.test(line)) {
      i++;
      continue;
    }

    // HTML comments are dropped.
    if (/^\s*<!--/.test(line)) {
      while (i < lines.length && !/-->/.test(lines[i])) i++;
      i++;
      continue;
    }

    let m;

    // fenced code
    if ((m = RE.fence.exec(line))) {
      const indent = m[1].length;
      const fenceCh = m[2][0];
      const fenceLen = m[2].length;
      const body = [];
      i++;
      const closeRe = new RegExp(`^ {0,3}${fenceCh === '`' ? '`' : '~'}{${fenceLen},}[ \\t]*$`);
      while (i < lines.length && !closeRe.test(lines[i])) {
        body.push(lines[i].replace(new RegExp(`^ {0,${indent}}`), ''));
        i++;
      }
      i++; // closing fence
      out.push(codeBlockHtml(body.join('\n'), m[3]));
      continue;
    }

    // heading
    if ((m = RE.heading.exec(line))) {
      out.push(ctx.heading(m[1].length, m[2].trim()));
      i++;
      continue;
    }

    // horizontal rule
    if (RE.hr.test(line)) {
      out.push('<hr>');
      i++;
      continue;
    }

    // blockquote / alert
    if (RE.quote.test(line)) {
      const body = [];
      while (i < lines.length) {
        const l = lines[i];
        if (RE.quote.test(l)) body.push(l.replace(/^ {0,3}> ?/, ''));
        else if (!RE.blank.test(l) && body.length && !RE.blank.test(body[body.length - 1]) && !interruptsParagraph(lines, i)) body.push(l);
        else break;
        i++;
      }
      const alert = /^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*$/i.exec(body[0] || '');
      if (alert) {
        const kind = alert[1].toLowerCase();
        const inner = parseBlocks(body.slice(1), ctx);
        out.push(
          `<div class="alert alert-${kind}" role="note"><p class="alert-title">${ALERT_TITLES[kind]}</p>${inner}</div>`,
        );
      } else {
        out.push(`<blockquote>${parseBlocks(body, ctx)}</blockquote>`);
      }
      continue;
    }

    // table
    if (isTableStart(lines, i)) {
      const head = splitRow(lines[i]);
      const aligns = splitRow(lines[i + 1]).map((c) => {
        const l = c.startsWith(':');
        const r = c.endsWith(':');
        return l && r ? 'center' : r ? 'right' : l ? 'left' : '';
      });
      i += 2;
      const rows = [];
      while (i < lines.length && !RE.blank.test(lines[i]) && lines[i].includes('|')) {
        rows.push(splitRow(lines[i]));
        i++;
      }
      const cell = (tag, txt, k) =>
        `<${tag}${aligns[k] ? ` style="text-align:${aligns[k]}"` : ''}>${renderInline(txt, ctx)}</${tag}>`;
      out.push(
        `<div class="table-wrap"><table><thead><tr>${head.map((c, k) => cell('th', c, k)).join('')}</tr></thead>` +
          `<tbody>${rows
            .map((r) => `<tr>${head.map((_, k) => cell('td', r[k] ?? '', k)).join('')}</tr>`)
            .join('')}</tbody></table></div>`,
      );
      continue;
    }

    // list
    if ((m = RE.list.exec(line))) {
      const ordered = /\d/.test(m[2][0]);
      const markerChar = ordered ? m[2].slice(-1) : m[2];
      const start = ordered ? parseInt(m[2], 10) : 1;
      const items = [];
      let loose = false;
      while (i < lines.length) {
        const lm = RE.list.exec(lines[i]);
        if (!lm || RE.hr.test(lines[i])) break;
        const lordered = /\d/.test(lm[2][0]);
        const lchar = lordered ? lm[2].slice(-1) : lm[2];
        if (lordered !== ordered || lchar !== markerChar) break;
        const indent = lm[1].length;
        const spaces = lm[0].length - indent - lm[2].length;
        const rest = lines[i].slice(lm[0].length);
        const hasContent = rest.trim() !== '';
        const contentOffset = indent + lm[2].length + (!hasContent || spaces > 4 ? 1 : spaces);
        const body = [spaces > 4 ? lines[i].slice(contentOffset) : rest];
        let j = i + 1;
        while (j < lines.length) {
          const l = lines[j];
          if (RE.blank.test(l)) {
            let k = j + 1;
            while (k < lines.length && RE.blank.test(lines[k])) k++;
            if (k < lines.length && indentOf(lines[k]) >= contentOffset) {
              body.push('');
              j++;
              continue;
            }
            break;
          }
          if (indentOf(l) >= contentOffset) {
            body.push(l.slice(contentOffset));
            j++;
            continue;
          }
          const prev = body[body.length - 1];
          if (prev !== '' && !RE.list.test(l) && !interruptsParagraph(lines, j) && !isTableStart(lines, j)) {
            body.push(l.trim());
            j++;
            continue;
          }
          break;
        }
        const trimmed = [...body];
        while (trimmed.length && trimmed[trimmed.length - 1] === '') trimmed.pop();
        if (trimmed.includes('') && trimmed.some((x, idx) => x === '' && idx > 0 && idx < trimmed.length - 1)) loose = true;
        items.push(trimmed);
        // look past blank lines for the next sibling
        let k = j;
        while (k < lines.length && RE.blank.test(lines[k])) k++;
        if (k > j) {
          const nm = k < lines.length ? RE.list.exec(lines[k]) : null;
          const sameType =
            nm && /\d/.test(nm[2][0]) === ordered && (ordered ? nm[2].slice(-1) : nm[2]) === markerChar && nm[1].length <= 3;
          if (sameType && !RE.hr.test(lines[k])) {
            loose = true;
            i = k;
            continue;
          }
          i = j;
          break;
        }
        i = j;
      }
      const tag = ordered ? 'ol' : 'ul';
      const startAttr = ordered && start !== 1 ? ` start="${start}"` : '';
      out.push(
        `<${tag}${startAttr}>${items.map((b) => `<li>${parseBlocks(b, ctx, !loose)}</li>`).join('')}</${tag}>`,
      );
      continue;
    }

    // paragraph
    const para = [line];
    i++;
    let setext = 0;
    while (i < lines.length && !RE.blank.test(lines[i])) {
      const u = /^ {0,3}(=+|-+)[ \t]*$/.exec(lines[i]);
      if (u) {
        setext = u[1][0] === '=' ? 1 : 2;
        i++;
        break;
      }
      if (interruptsParagraph(lines, i) || isTableStart(lines, i)) break;
      para.push(lines[i]);
      i++;
    }
    if (setext) {
      out.push(ctx.heading(setext, para.map((l) => l.trim()).join(' ')));
      continue;
    }
    const joined = para
      .map((l, idx) => {
        const t = l.replace(/^\s+/, '');
        if (idx === para.length - 1) return t.replace(/\s+$/, '');
        if (/( {2,}|\\)$/.test(t)) return t.replace(/( {2,}|\\)$/, '') + '\u0001';
        return t.replace(/\s+$/, '');
      })
      .join('\n');
    const inner = renderInline(joined, ctx);
    out.push(tight ? inner : `<p>${inner}</p>`);
  }
  return tight ? out.join('\n').replace(/\n(?=<(?:ul|ol)[ >])/g, '') : out.join('\n');
}

/**
 * Render markdown to HTML.
 *
 * @param {string} src
 * @param {object} opts
 * @param {(href:string)=>string} [opts.resolveLink]
 * @param {(src:string)=>string} [opts.resolveImage]
 * @param {number} [opts.offset]       added to every heading level
 * @param {boolean} [opts.stripTitle]  drop the first H1
 * @param {Map<string,number>} [opts.usedIds] shared across a page for unique ids
 * @param {Array} [opts.headings]      receives {level,id,text}
 */
export function renderMarkdown(src, opts = {}) {
  const usedIds = opts.usedIds ?? new Map();
  const headings = opts.headings ?? [];
  let firstH1Pending = !!opts.stripTitle;
  const ctx = {
    resolveLink: opts.resolveLink,
    resolveImage: opts.resolveImage,
    heading(level, raw) {
      const html = renderInline(raw, ctx);
      if (level === 1 && firstH1Pending) {
        firstH1Pending = false;
        return '';
      }
      const text = stripTags(html).trim();
      const finalLevel = Math.min(6, level + (opts.offset ?? 0));
      let id = opts.idForHeading?.(text, level) ?? slugify(text);
      const n = usedIds.get(id) ?? 0;
      usedIds.set(id, n + 1);
      if (n > 0) id = `${id}-${n}`;
      headings.push({ level: finalLevel, id, text });
      return `<h${finalLevel} id="${escapeHtml(id)}"><a class="anchor" href="#${escapeHtml(id)}" aria-label="Link to this section">#</a>${html}</h${finalLevel}>`;
    },
  };
  const lines = String(src).replace(/\r\n?/g, '\n').replace(/\t/g, '    ').split('\n');
  const html = parseBlocks(lines, ctx);
  return { html, headings };
}

// ---------------------------------------------------------------------------
// Markdown helpers for extracting parts of source documents
// ---------------------------------------------------------------------------

const normTitle = (t) => String(t).replace(/[`*_]/g, '').replace(/\s+/g, ' ').trim().toLowerCase();

function eachHeading(md, fn) {
  const lines = md.replace(/\r\n?/g, '\n').split('\n');
  let fence = null;
  for (let i = 0; i < lines.length; i++) {
    const f = RE.fence.exec(lines[i]);
    if (f) {
      if (!fence) fence = f[2][0];
      else if (fence === f[2][0] && /^\s*(`{3,}|~{3,})\s*$/.test(lines[i])) fence = null;
      continue;
    }
    if (fence) continue;
    const h = RE.heading.exec(lines[i]);
    if (h) fn(i, h[1].length, h[2].trim(), lines);
  }
  return lines;
}

/** Cut the section with the given heading text. Returns { section, rest } (section includes its heading). */
export function cutSection(md, title) {
  let startIdx = -1;
  let startLevel = 0;
  let endIdx = -1;
  const lines = eachHeading(md, (i, level, text) => {
    if (startIdx < 0) {
      if (normTitle(text) === normTitle(title)) {
        startIdx = i;
        startLevel = level;
      }
    } else if (endIdx < 0 && level <= startLevel) endIdx = i;
  });
  if (startIdx < 0) throw new Error(`Section not found: ${title}`);
  if (endIdx < 0) endIdx = lines.length;
  return {
    section: lines.slice(startIdx, endIdx).join('\n'),
    rest: [...lines.slice(0, startIdx), ...lines.slice(endIdx)].join('\n'),
  };
}

export function extractSection(md, title, { includeHeading = false, exclude = [] } = {}) {
  let { section } = cutSection(md, title);
  for (const ex of exclude) section = cutSection(section, ex).rest;
  if (!includeHeading) section = section.split('\n').slice(1).join('\n');
  return section;
}
