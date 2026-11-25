/* SpectreOS site behaviour: theme, drawer, copy buttons, search, TOC spy, live GitHub releases. */
(function () {
  'use strict';
  var root = document.documentElement;
  var BASE = root.getAttribute('data-base') || '/';
  var $ = function (s, c) { return (c || document).querySelector(s); };
  var $$ = function (s, c) { return Array.prototype.slice.call((c || document).querySelectorAll(s)); };

  /* ---- theme ---------------------------------------------------------- */
  var themeBtn = $('#theme-btn');
  if (themeBtn) {
    themeBtn.addEventListener('click', function () {
      var next = root.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
      root.setAttribute('data-theme', next);
      try { localStorage.setItem('spectreos-theme', next); } catch (e) { /* storage unavailable */ }
      var meta = $('meta[name="theme-color"]');
      if (meta) meta.setAttribute('content', next === 'light' ? '#f5f8fb' : '#07090d');
    });
  }

  /* ---- mobile drawer -------------------------------------------------- */
  var sidebar = $('#sidebar');
  var menuBtn = $('#menu-btn');
  var scrim = $('#scrim');
  if (sidebar) document.body.classList.add('has-sidebar');
  function setDrawer(open) {
    if (!sidebar) return;
    sidebar.classList.toggle('open', open);
    if (scrim) scrim.hidden = !open;
    if (menuBtn) menuBtn.setAttribute('aria-expanded', String(open));
  }
  if (menuBtn) menuBtn.addEventListener('click', function () { setDrawer(!sidebar.classList.contains('open')); });
  if (scrim) scrim.addEventListener('click', function () { setDrawer(false); });
  if (sidebar) sidebar.addEventListener('click', function (e) { if (e.target.closest('a')) setDrawer(false); });

  /* ---- clipboard ------------------------------------------------------ */
  function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) return navigator.clipboard.writeText(text);
    return new Promise(function (resolve, reject) {
      var ta = document.createElement('textarea');
      ta.value = text; ta.setAttribute('readonly', ''); ta.style.position = 'fixed'; ta.style.opacity = '0';
      document.body.appendChild(ta); ta.select();
      try { document.execCommand('copy') ? resolve() : reject(); } catch (e) { reject(e); }
      document.body.removeChild(ta);
    });
  }
  function flash(btn, ok, label) {
    var old = btn.getAttribute('data-label') || btn.textContent;
    btn.setAttribute('data-label', old);
    if (btn.classList.contains('copy-btn')) btn.textContent = ok ? 'Copied' : 'Press Ctrl+C';
    btn.classList.add('copied');
    setTimeout(function () { btn.classList.remove('copied'); if (btn.classList.contains('copy-btn')) btn.textContent = old; }, 1600);
  }
  document.addEventListener('click', function (e) {
    var btn = e.target.closest('.copy-btn, .copy-chip');
    if (!btn) return;
    var text;
    if (btn.classList.contains('copy-chip')) text = btn.getAttribute('data-copy');
    else {
      var code = btn.closest('figure.code').querySelector('code');
      text = code.textContent;
    }
    copyText(text).then(function () { flash(btn, true); }, function () { flash(btn, false); });
  });

  /* ---- TOC scroll-spy ------------------------------------------------- */
  var tocLinks = $$('.toc a');
  if (tocLinks.length && 'IntersectionObserver' in window) {
    var map = {};
    tocLinks.forEach(function (a) { map[decodeURIComponent(a.getAttribute('href').slice(1))] = a; });
    var heads = Object.keys(map).map(function (id) { return document.getElementById(id); }).filter(Boolean);
    var current = null;
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (en) {
        if (en.isIntersecting) {
          if (current) current.classList.remove('active');
          current = map[en.target.id];
          if (current) { current.classList.add('active'); current.scrollIntoView && current.scrollIntoView({ block: 'nearest' }); }
        }
      });
    }, { rootMargin: '-70px 0px -70% 0px' });
    heads.forEach(function (h) { io.observe(h); });
  }

  /* ---- search --------------------------------------------------------- */
  var modal = $('#search-modal');
  var input = $('#search-input');
  var list = $('#search-results');
  var index = null;
  var loading = null;
  var active = 0;
  var shown = [];
  var lastFocus = null;

  function loadIndex() {
    if (index) return Promise.resolve(index);
    if (!loading) {
      loading = fetch(BASE + 'search-index.json').then(function (r) { return r.json(); }).then(function (j) {
        index = j.map(function (e) { return { e: e, t: e.t.toLowerCase(), p: e.p.toLowerCase(), x: e.x.toLowerCase() }; });
        return index;
      }).catch(function () { index = []; return index; });
    }
    return loading;
  }
  function esc(s) { return s.replace(/[&<>"]/g, function (c) { return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]; }); }
  function escRe(s) { return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); }
  function mark(text, terms) {
    var out = esc(text);
    terms.forEach(function (t) { out = out.replace(new RegExp('(' + escRe(esc(t)) + ')', 'ig'), '<mark>$1</mark>'); });
    return out;
  }
  function snippet(x, terms) {
    var low = x.toLowerCase(), at = -1;
    for (var i = 0; i < terms.length && at < 0; i++) at = low.indexOf(terms[i]);
    var start = Math.max(0, at - 50);
    var s = x.slice(start, start + 160);
    return (start > 0 ? '… ' : '') + s + (start + 160 < x.length ? ' …' : '');
  }
  function search(q) {
    var terms = q.toLowerCase().split(/\s+/).filter(Boolean);
    if (!terms.length) return [];
    var res = [];
    index.forEach(function (it) {
      var score = 0;
      for (var i = 0; i < terms.length; i++) {
        var t = terms[i], s = 0;
        if (it.t === t) s += 12; else if (it.t.indexOf(t) === 0) s += 8; else if (it.t.indexOf(t) > -1) s += 6;
        if (it.p.indexOf(t) > -1) s += 2;
        if (it.x.indexOf(t) > -1) s += 1;
        if (!s) return;
        score += s;
      }
      res.push({ it: it.e, score: score });
    });
    res.sort(function (a, b) { return b.score - a.score; });
    return res.slice(0, 12).map(function (r) { return r.it; });
  }
  function render(q) {
    var terms = q.toLowerCase().split(/\s+/).filter(Boolean);
    shown = search(q);
    active = 0;
    if (!q.trim()) { list.innerHTML = ''; return; }
    if (!shown.length) { list.innerHTML = '<li class="search-empty">No results for “' + esc(q) + '”</li>'; return; }
    list.innerHTML = shown.map(function (e, i) {
      return '<li role="option" id="sr-' + i + '" aria-selected="' + (i === 0) + '"><a href="' + BASE + e.u + '">' +
        '<span class="sr-page">' + esc(e.p) + '</span><span class="sr-title">' + mark(e.t, terms) + '</span>' +
        '<span class="sr-snippet">' + mark(snippet(e.x, terms), terms) + '</span></a></li>';
    }).join('');
  }
  function setActive(i) {
    if (!shown.length) return;
    active = (i + shown.length) % shown.length;
    $$('li[role="option"]', list).forEach(function (li, n) { li.setAttribute('aria-selected', String(n === active)); });
    var cur = $('#sr-' + active); if (cur) cur.scrollIntoView({ block: 'nearest' });
  }
  function openSearch() {
    if (!modal) return;
    lastFocus = document.activeElement;
    modal.hidden = false; document.body.style.overflow = 'hidden';
    input.value = ''; list.innerHTML = '';
    loadIndex(); input.focus();
  }
  function closeSearch() {
    if (!modal || modal.hidden) return;
    modal.hidden = true; document.body.style.overflow = '';
    if (lastFocus && lastFocus.focus) lastFocus.focus();
  }
  if (modal) {
    var sb = $('#search-btn');
    if (sb) sb.addEventListener('click', openSearch);
    modal.addEventListener('click', function (e) { if (e.target.hasAttribute('data-close') || e.target.closest('a')) closeSearch(); });
    input.addEventListener('input', function () { loadIndex().then(function () { render(input.value); }); });
    input.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowDown') { e.preventDefault(); setActive(active + 1); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); setActive(active - 1); }
      else if (e.key === 'Enter' && shown[active]) { e.preventDefault(); window.location.href = BASE + shown[active].u; }
    });
    document.addEventListener('keydown', function (e) {
      var tag = (e.target.tagName || '').toLowerCase();
      var typing = tag === 'input' || tag === 'textarea' || tag === 'select' || e.target.isContentEditable;
      if ((e.key === 'k' || e.key === 'K') && (e.ctrlKey || e.metaKey)) { e.preventDefault(); modal.hidden ? openSearch() : closeSearch(); }
      else if (e.key === '/' && !typing && !e.ctrlKey && !e.metaKey && !e.altKey) { e.preventDefault(); openSearch(); }
      else if (e.key === 'Escape') { closeSearch(); setDrawer(false); }
    });
  }

  /* ---- live GitHub releases ------------------------------------------ */
  var dataEl = $('#spectreos-data');
  if (dataEl) {
    var data = JSON.parse(dataEl.textContent);
    var comps = data.components;
    var CACHE_KEY = 'spectreos-releases-v1';
    var fmtDate = function (s) { try { return new Date(s).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' }); } catch (e) { return ''; } };
    var fmtSize = function (n) { return n > 1048576 ? (n / 1048576).toFixed(1) + ' MB' : Math.max(1, Math.round(n / 1024)) + ' KB'; };
    var compOf = function (tag) {
      for (var i = 0; i < comps.length; i++) if (tag.indexOf(comps[i].tagPrefix) === 0) return comps[i];
      return null;
    };
    var fetchReleases = function () {
      try {
        var c = JSON.parse(sessionStorage.getItem(CACHE_KEY) || 'null');
        if (c && Date.now() - c.t < 600000) return Promise.resolve(c.d);
      } catch (e) { /* ignore */ }
      return fetch(data.api + '?per_page=100', { headers: { Accept: 'application/vnd.github+json' } }).then(function (r) {
        if (!r.ok) throw new Error('GitHub API ' + r.status);
        return r.json();
      }).then(function (d) {
        var slim = d.filter(function (r) { return !r.draft; }).map(function (r) {
          return { tag: r.tag_name, name: r.name, url: r.html_url, date: r.published_at || r.created_at, pre: r.prerelease,
            assets: (r.assets || []).map(function (a) { return { name: a.name, url: a.browser_download_url, size: a.size }; }) };
        });
        try { sessionStorage.setItem(CACHE_KEY, JSON.stringify({ t: Date.now(), d: slim })); } catch (e) { /* ignore */ }
        return slim;
      });
    };
    var el = function (tag, cls, text) { var n = document.createElement(tag); if (cls) n.className = cls; if (text != null) n.textContent = text; return n; };
    var link = function (href, cls, text) { var a = el('a', cls, text); a.href = href; a.rel = 'noopener'; return a; };
    var fallbackLink = function (c) { return data.repo + '/releases?q=' + encodeURIComponent(c.tagPrefix); };

    fetchReleases().then(function (releases) {
      var latest = {};
      releases.forEach(function (r) {
        var c = compOf(r.tag);
        if (c && !latest[c.id]) latest[c.id] = r;
      });
      $$('[data-live-for]').forEach(function (slot) {
        var c = comps.filter(function (x) { return x.id === slot.getAttribute('data-live-for'); })[0];
        if (!c) return;
        var r = latest[c.id];
        slot.textContent = '';
        if (!r) {
          var s = el('span', 'muted small', 'No GitHub release yet');
          slot.appendChild(s); slot.appendChild(document.createTextNode(' · '));
          slot.appendChild(link(fallbackLink(c), 'small', 'releases'));
          return;
        }
        var tag = link(r.url, 'live-tag', r.tag);
        slot.appendChild(tag);
        slot.appendChild(el('span', 'live-date', fmtDate(r.date)));
        if (slot.classList.contains('live-slot') && r.assets.length) {
          var row = el('div', 'dl-row');
          r.assets.slice(0, 3).forEach(function (a) { row.appendChild(link(a.url, 'dl', '↓ ' + a.name + ' · ' + fmtSize(a.size))); });
          slot.appendChild(row);
        }
      });
      var listEl = $('#release-list');
      if (listEl) {
        var filter = $('#release-filter');
        var draw = function () {
          var f = filter ? filter.value : '';
          listEl.textContent = '';
          var rows = releases.filter(function (r) { var c = compOf(r.tag); return !f || (c && c.id === f); });
          if (!rows.length) {
            var empty = el('div', 'release-empty');
            empty.appendChild(document.createTextNode('No GitHub releases published for this selection yet. '));
            empty.appendChild(link(data.repo + '/releases', '', 'Browse all releases on GitHub'));
            empty.appendChild(document.createTextNode('.'));
            listEl.appendChild(empty); return;
          }
          rows.forEach(function (r) {
            var c = compOf(r.tag);
            var card = el('article', 'release');
            var top = el('div', 'release-top');
            top.appendChild(link(r.url, 'release-tag', r.tag));
            if (c) top.appendChild(el('span', 'release-name', c.name));
            if (r.pre) top.appendChild(el('span', 'pill', 'pre-release'));
            top.appendChild(el('span', 'release-meta', fmtDate(r.date)));
            card.appendChild(top);
            if (r.assets.length) {
              var row = el('div', 'dl-row');
              r.assets.forEach(function (a) { row.appendChild(link(a.url, 'dl', '↓ ' + a.name + ' · ' + fmtSize(a.size))); });
              card.appendChild(row);
            }
            listEl.appendChild(card);
          });
        };
        draw();
        if (filter) filter.addEventListener('change', draw);
      }
    }).catch(function () {
      $$('[data-live-for]').forEach(function (slot) {
        var c = comps.filter(function (x) { return x.id === slot.getAttribute('data-live-for'); })[0];
        if (!c) return;
        slot.textContent = '';
        slot.appendChild(link(fallbackLink(c), 'small', 'View releases on GitHub'));
      });
      var listEl = $('#release-list');
      if (listEl) {
        listEl.textContent = '';
        var box = el('div', 'release-empty');
        box.appendChild(document.createTextNode('Could not reach the GitHub API (offline or rate-limited). '));
        box.appendChild(link(data.repo + '/releases', '', 'Open GitHub Releases'));
        box.appendChild(document.createTextNode(' to browse and download.'));
        listEl.appendChild(box);
      }
    });
  }
})();
