import { paintPacket } from '/bhoo-web/canvas-painter.js';
import { heroicon, hydrateHeroicons } from './heroicons.js';

hydrateHeroicons();

const appCanvas = document.getElementById('appCanvas');
const overlayCanvas = document.getElementById('overlayCanvas');
const feedbackCanvas = document.getElementById('feedbackCanvas');
const ctx = appCanvas.getContext('2d', { alpha: false });
const octx = overlayCanvas.getContext('2d');
const fctx = feedbackCanvas.getContext('2d');
const encoder = new TextEncoder();
const decoder = new TextDecoder();

const elBtnAppPicker = document.getElementById('btnAppPicker');
const elAppPickerIcon = document.getElementById('appPickerIcon');
const elAppPickerName = document.getElementById('appPickerName');
const elAppPickerModal = document.getElementById('appPickerModal');
const elBtnCloseAppPicker = document.getElementById('btnCloseAppPicker');
const elAppPickerSearch = document.getElementById('appPickerSearch');
const elAppCardGrid = document.getElementById('appCardGrid');
const elFilesModeWrap = document.getElementById('filesModeWrap');
const elFilesMode = document.getElementById('filesMode');
const elNotifControlsWrap = document.getElementById('notifControlsWrap');
const elNotifMode = document.getElementById('notifMode');
const elBtnNotifNormal = document.getElementById('btnNotifNormal');
const elBtnNotifActions = document.getElementById('btnNotifActions');
const elBtnNotifCritical = document.getElementById('btnNotifCritical');
const elBtnNotifClear = document.getElementById('btnNotifClear');
const elViewportPreset = document.getElementById('viewportPreset');
const elCustomSizeWrap = document.getElementById('customSizeWrap');
const elCustomW = document.getElementById('customW');
const elCustomH = document.getElementById('customH');
const elBtnRotate = document.getElementById('btnRotate');
const elBtnBack = document.getElementById('btnBack');
const elBtnReset = document.getElementById('btnReset');
const elChkDark = document.getElementById('chkDark');
const elChkTouch = document.getElementById('chkTouch');
const elChkReduced = document.getElementById('chkReduced');
const elChkBounds = document.getElementById('chkBounds');
const elStageWrap = document.getElementById('stageWrap');
const elStageScroll = document.getElementById('stageScroll');
const elBtnStageDraw = document.getElementById('btnStageDraw');
const elDrawModeHud = document.getElementById('drawModeHud');
const elBtnExitDrawMode = document.getElementById('btnExitDrawMode');
const elFeedbackPopover = document.getElementById('feedbackPopover');
const elFbDraftNum = document.getElementById('fbDraftNum');
const elFbDraftTarget = document.getElementById('fbDraftTarget');
const elBtnCancelFeedbackDraft = document.getElementById('btnCancelFeedbackDraft');
const elFeedbackInput = document.getElementById('feedbackInput');
const elBtnSaveFeedbackDraft = document.getElementById('btnSaveFeedbackDraft');
const elFeedbackToast = document.getElementById('feedbackToast');
const elFeedbackDock = document.getElementById('feedbackDock');
const elFeedbackCountBadge = document.getElementById('feedbackCountBadge');
const elBtnDockDraw = document.getElementById('btnDockDraw');
const elFeedbackChipList = document.getElementById('feedbackChipList');
const elBtnCopyAllAiText = document.getElementById('btnCopyAllAiText');
const elBtnCopyAllAiImg = document.getElementById('btnCopyAllAiImg');
const elBtnDownloadAllAiImg = document.getElementById('btnDownloadAllAiImg');
const elBtnClearFeedbacks = document.getElementById('btnClearFeedbacks');
const elFeedbackPreviewModal = document.getElementById('feedbackPreviewModal');
const elFbPreviewNum = document.getElementById('fbPreviewNum');
const elFbPreviewHeading = document.getElementById('fbPreviewHeading');
const elFbPreviewMeta = document.getElementById('fbPreviewMeta');
const elFbPreviewImg = document.getElementById('fbPreviewImg');
const elFbPreviewPrompt = document.getElementById('fbPreviewPrompt');
const elBtnPreviewCopyText = document.getElementById('btnPreviewCopyText');
const elBtnPreviewCopyImg = document.getElementById('btnPreviewCopyImg');
const elBtnPreviewDownloadImg = document.getElementById('btnPreviewDownloadImg');
const elBtnCloseFbPreview = document.getElementById('btnCloseFbPreview');
const elWorkbench = document.querySelector('.workbench');
const elInspectorSplitter = document.getElementById('inspectorSplitter');
const elStateLogSplitter = document.getElementById('stateLogSplitter');
const elInspectorPane = document.getElementById('inspectorPane');
const elBuildStatus = document.getElementById('buildStatus');

// Notion-style Settings Modal elements
const elBtnSettings = document.getElementById('btnSettings');
const elSettingsModal = document.getElementById('settingsModal');
const elBtnCloseSettings = document.getElementById('btnCloseSettings');
const elSettingsModalHeading = document.getElementById('settingsModalHeading');
const elChkDotGrid = document.getElementById('chkDotGrid');
const elChkModalTestBar = document.getElementById('chkModalTestBar');
const elChkModalInspector = document.getElementById('chkModalInspector');
const elChkModalBounds = document.getElementById('chkModalBounds');
const elSettingsFontMeta = document.getElementById('settingsFontMeta');
const elBtnResetLayout = document.getElementById('btnResetLayout');
const elBtnResetTests = document.getElementById('btnResetTests');
const elBtnClearAllLogs = document.getElementById('btnClearAllLogs');

// Quick Test Bar & Test Builder elements
const elQuickTestBar = document.getElementById('quickTestBar');
const elQuickTestPills = document.getElementById('quickTestPills');
const elBtnBarRecord = document.getElementById('btnBarRecord');
const elBarRecordLabel = document.getElementById('barRecordLabel');
const elBtnBarNewTest = document.getElementById('btnBarNewTest');
const elBtnBarRunAll = document.getElementById('btnBarRunAll');
const elBtnNewTest = document.getElementById('btnNewTest');
const elBtnRecordTest = document.getElementById('btnRecordTest');
const elTabRecordLabel = document.getElementById('tabRecordLabel');
const elBtnRunAllTests = document.getElementById('btnRunAllTests');
const elBtnExportTests = document.getElementById('btnExportTests');
const elBtnImportTests = document.getElementById('btnImportTests');
const elImportTestsFile = document.getElementById('importTestsFile');
const elTestEditorForm = document.getElementById('testEditorForm');
const elTestEditorTitle = document.getElementById('testEditorTitle');
const elBtnCloseTestEditor = document.getElementById('btnCloseTestEditor');
const elBtnCancelTest = document.getElementById('btnCancelTest');
const elTcName = document.getElementById('tcName');
const elTcApp = document.getElementById('tcApp');
const elTcPin = document.getElementById('tcPin');
const elTcReset = document.getElementById('tcReset');
const elTcAddStepType = document.getElementById('tcAddStepType');
const elBtnAddStep = document.getElementById('btnAddStep');
const elTcStepList = document.getElementById('tcStepList');
const elTestCaseList = document.getElementById('testCaseList');

// System Notification overlay & D-Bus Monitor elements
const elSysNotifStack = document.getElementById('sysNotifStack');
const elBusBadge = document.getElementById('busBadge');
const elBusPreset = document.getElementById('busPreset');
const elBtnSendBus = document.getElementById('btnSendBus');
const elBusNotifInputs = document.getElementById('busNotifInputs');
const elBusNotifApp = document.getElementById('busNotifApp');
const elBusNotifTitle = document.getElementById('busNotifTitle');
const elBusNotifBody = document.getElementById('busNotifBody');
const elBusFilterKind = document.getElementById('busFilterKind');
const elBtnClearBus = document.getElementById('btnClearBus');
const elBusList = document.getElementById('busList');

// Telemetry & Inspector elements
const elMetaDims = document.getElementById('metaDims');
const elMetaPointer = document.getElementById('metaPointer');
const elMetaFps = document.getElementById('metaFps');
const elBtnReloadWasm = document.getElementById('btnReloadWasm');
const elMWasm = document.getElementById('mWasm');
const elMPaint = document.getElementById('mPaint');
const elMCmds = document.getElementById('mCmds');
const elMNodes = document.getElementById('mNodes');
const elMBytes = document.getElementById('mBytes');
const elMLoop = document.getElementById('mLoop');
const elHoverNode = document.getElementById('hoverNode');
const elAppStateJson = document.getElementById('appStateJson');
const elActionLog = document.getElementById('actionLog');
const elBtnClearLog = document.getElementById('btnClearLog');
const elSemFilter = document.getElementById('semFilter');
const elSemList = document.getElementById('semList');
const elStageLoader = document.getElementById('stageLoader');
const elLoaderSpinner = document.getElementById('loaderSpinner');
const elLoaderTitle = document.getElementById('loaderTitle');
const elLoaderSub = document.getElementById('loaderSub');
const elDlLabel = document.getElementById('dlLabel');
const elDlBarFill = document.getElementById('dlBarFill');
const elLoadLabel = document.getElementById('loadLabel');
const elLoadBarFill = document.getElementById('loadBarFill');
const elBtnLoaderRetry = document.getElementById('btnLoaderRetry');

let engine = null;
let frameTrapped = false;
let apps = [];
let appCatalog = new Map();
let activeAppIdx = 0;

const APP_VISUALS = {
  files: {
    icon: 'folder',
    tone: 'app-tone-files',
    name: 'Files',
    genericName: 'File Manager',
    comment: 'Browse and organize local files, media filters, and file chooser portals on SpectreOS.',
    desktopId: 'org.spectreos.Files',
  },
  settings: {
    icon: 'cog-6-tooth',
    tone: 'app-tone-settings',
    name: 'SpectreOS Settings',
    genericName: 'System Settings',
    comment: 'Configure SpectreOS display, notifications, lock screen, network, and device preferences.',
    desktopId: 'org.spectreos.Settings',
  },
  calendar: {
    icon: 'calendar-days',
    tone: 'app-tone-calendar',
    name: 'Calendar',
    genericName: 'Calendar & Schedule',
    comment: 'Manage local calendars, recurring events, reminders, and iCalendar files on SpectreOS.',
    desktopId: 'org.spectreos.Calendar',
  },
  keyboard: {
    icon: 'command-line',
    tone: 'app-tone-keyboard',
    name: 'SpectreOS Keyboard',
    genericName: 'On-Screen Keyboard',
    comment: 'Default SpectreOS Wayland on-screen keyboard with English, Persian (Shabnam), Terminal, and Emoji layouts.',
    desktopId: 'org.spectreos.Keyboard',
  },
  notifications: {
    icon: 'bell-alert',
    tone: 'app-tone-notifications',
    name: 'Notifications',
    genericName: 'Wisp System UI & Alerts',
    comment: 'Test Wisp heads-up banners, notification shade, lock screen alerts, and Freedesktop D-Bus actions.',
    desktopId: 'org.spectreos.Wisp',
  },
};

function getAppMeta(slug) {
  const visual = APP_VISUALS[slug] || {
    icon: 'squares-2x2',
    tone: 'app-tone-files',
    name: slug ? slug[0].toUpperCase() + slug.slice(1) : 'App',
    genericName: 'SpectreOS App',
    comment: 'Native libbhoo application.',
    desktopId: `org.spectreos.${slug ? slug[0].toUpperCase() + slug.slice(1) : 'App'}`,
  };
  const live = appCatalog.get(slug) || {};
  const localTestCount = testCases.filter(t => t.app === slug).length;
  return {
    slug,
    icon: visual.icon,
    tone: visual.tone,
    name: live.name || visual.name,
    genericName: live.genericName || visual.genericName,
    comment: live.comment || visual.comment,
    desktopId: live.desktopId || visual.desktopId,
    categories: Array.isArray(live.categories) ? live.categories : [],
    keywords: Array.isArray(live.keywords) ? live.keywords : [],
    testCount: Math.max(live.testCount || 0, localTestCount),
  };
}

async function loadAppCatalog() {
  try {
    const resp = await fetch('/__debug/apps', { cache: 'no-store' });
    if (!resp.ok) return;
    const list = await resp.json();
    if (!Array.isArray(list)) return;
    appCatalog.clear();
    for (const item of list) {
      if (item && item.slug) appCatalog.set(item.slug, item);
    }
    updateAppPickerButton();
    if (!elAppPickerModal.hidden) renderAppCards();
  } catch {}
}

function updateAppPickerButton() {
  const slug = apps[activeAppIdx];
  if (!slug) {
    elAppPickerName.textContent = 'Loading…';
    return;
  }
  const meta = getAppMeta(slug);
  elAppPickerName.textContent = meta.name;
  elAppPickerIcon.className = `app-picker-icon ${meta.tone}`;
  elAppPickerIcon.innerHTML = heroicon(meta.icon);
}

function renderAppCards() {
  const q = (elAppPickerSearch.value || '').trim().toLowerCase();
  elAppCardGrid.innerHTML = '';

  const matched = [];
  apps.forEach((slug, idx) => {
    const meta = getAppMeta(slug);
    const hay = `${slug} ${meta.name} ${meta.genericName} ${meta.comment} ${meta.desktopId} ${meta.keywords.join(' ')} ${meta.categories.join(' ')}`.toLowerCase();
    if (!q || hay.includes(q)) matched.push({ slug, idx, meta });
  });

  if (!matched.length) {
    elAppCardGrid.innerHTML = `<div class="app-card-empty">No apps match "${escapeHtml(q)}".</div>`;
    return;
  }

  for (const { idx, meta } of matched) {
    const isActive = idx === activeAppIdx;
    const card = document.createElement('button');
    card.type = 'button';
    card.className = `app-card${isActive ? ' active' : ''}`;
    card.setAttribute('role', 'option');
    card.setAttribute('aria-selected', isActive ? 'true' : 'false');
    card.dataset.appIndex = String(idx);
    card.innerHTML = `
      <div class="app-card-top">
        <div class="app-card-identity">
          <span class="app-card-icon ${meta.tone}">${heroicon(meta.icon)}</span>
          <div class="app-card-titles">
            <span class="app-card-name">${escapeHtml(meta.name)}</span>
            <span class="app-card-generic">${escapeHtml(meta.genericName)}</span>
          </div>
        </div>
      </div>
      <p class="app-card-desc">${escapeHtml(meta.comment)}</p>
      <div class="app-card-meta">
        <span class="app-chip mono">${escapeHtml(meta.desktopId)}</span>
        <span class="app-chip">${heroicon('beaker')}<span>${meta.testCount} ${meta.testCount === 1 ? 'test' : 'tests'}</span></span>
      </div>
    `;
    card.addEventListener('click', () => {
      selectApp(idx);
      closeAppPickerModal();
    });
    elAppCardGrid.append(card);
  }
}

function openAppPickerModal() {
  if (!elSettingsModal.hidden) closeSettingsModal();
  elAppPickerSearch.value = '';
  renderAppCards();
  elAppPickerModal.hidden = false;
  requestAnimationFrame(() => elAppPickerSearch.focus());
}

function closeAppPickerModal() {
  elAppPickerModal.hidden = true;
}
let logicalW = 400;
let logicalH = 832;
let raf = 0;
let wakeTimer = 0;
let queue = [];
let pointer = [NaN, NaN];
let pointerDownPos = null;
let dragMoved = false;
let nodes = [];
let highlightNodeId = null;
let editor = null;
let activeFieldId = null;
let loadedVersion = null;
let availableVersion = null;
let buildReady = false;
let lastFrameStamp = 0;
let lastStateHash = '';
let lastParsedState = {};
let lastSemRender = 0;
let loaderHideTimer = 0;
let isLoadingWasm = false;
let activeTab = 'state';

// System Notification & D-Bus state
const busEvents = [];
let unreadBusCount = 0;
const activeOverlayNotifs = new Map();

// Test Case Manager + Recorder state
const TESTS_STORAGE_KEY = 'bhooDebug.testCases.v1';
let testCases = [];
const testResults = new Map(); // id -> { status: 'pass'|'fail'|'running', detail: string }
let editingTestId = null;
let editorSteps = [];
let isRecording = false;
let recordedSteps = [];
let runningTests = false;

const BUILTIN_TESTS = [
  {
    id: 'builtin-files-images',
    name: 'Images Picker',
    app: 'files',
    pinned: true,
    resetBefore: true,
    builtin: true,
    steps: [
      { type: 'filesMode', value: '2' },
      { type: 'wait', ms: 60 },
      { type: 'assertState', path: 'picker', expected: 'true' },
    ],
  },
  {
    id: 'builtin-files-saveas',
    name: 'Save As Portal',
    app: 'files',
    pinned: true,
    resetBefore: true,
    builtin: true,
    steps: [
      { type: 'dbus', preset: 'portal-save', value: 'report.txt' },
      { type: 'wait', ms: 60 },
      { type: 'assertState', path: 'picker', expected: 'true' },
    ],
  },
  {
    id: 'builtin-settings-notify',
    name: 'System Notify + History',
    app: 'settings',
    pinned: true,
    resetBefore: false,
    builtin: true,
    steps: [
      { type: 'pushNotif', app: 'Settings', title: 'Test System Alert', body: 'Fired from Test Case button', urgency: 1 },
      { type: 'wait', ms: 80 },
      { type: 'assertState', path: 'notifHistoryCount', expected: '>=1' },
    ],
  },
  {
    id: 'builtin-settings-wifi-dbus',
    name: 'D-Bus Wi-Fi On',
    app: 'settings',
    pinned: true,
    resetBefore: false,
    builtin: true,
    steps: [
      { type: 'dbus', preset: 'nm-wifi-on', value: 'true' },
      { type: 'wait', ms: 60 },
      { type: 'assertState', path: 'wifiEnabled', expected: 'true' },
    ],
  },
  {
    id: 'builtin-kb-farsi-landscape',
    name: 'Farsi Landscape',
    app: 'keyboard',
    pinned: true,
    resetBefore: false,
    builtin: true,
    steps: [
      { type: 'viewport', w: 832, h: 400 },
      { type: 'kbLayout', value: 'ir' },
      { type: 'wait', ms: 60 },
      { type: 'assertState', path: 'layout', expected: 'ir' },
    ],
  },
  {
    id: 'builtin-kb-terminal',
    name: 'Terminal Layout',
    app: 'keyboard',
    pinned: true,
    resetBefore: false,
    builtin: true,
    steps: [
      { type: 'viewport', w: 400, h: 832 },
      { type: 'kbLayout', value: 'terminal' },
      { type: 'wait', ms: 60 },
      { type: 'assertState', path: 'layout', expected: 'terminal' },
    ],
  },
  {
    id: 'builtin-notif-critical',
    name: 'Critical Banner',
    app: 'notifications',
    pinned: true,
    resetBefore: true,
    builtin: true,
    steps: [
      { type: 'notifMode', value: '0' },
      { type: 'pushNotif', app: 'System Security', title: 'Critical Thermal Alert', body: 'Battery reached 46 °C', urgency: 2 },
      { type: 'wait', ms: 120 },
      { type: 'assertState', path: 'mode', expected: 'heads_up_pinned' },
    ],
  },
  {
    id: 'builtin-notif-shade',
    name: 'Open Shade',
    app: 'notifications',
    pinned: true,
    resetBefore: true,
    builtin: true,
    steps: [
      { type: 'notifMode', value: '1' },
      { type: 'wait', ms: 80 },
      { type: 'assertState', path: 'shadeStage', expected: '1' },
    ],
  },
  {
    id: 'builtin-cal-reminder',
    name: 'Reminder Notify',
    app: 'calendar',
    pinned: true,
    resetBefore: false,
    builtin: true,
    steps: [
      { type: 'pushNotif', app: 'Calendar', title: 'Design Review in 10m', body: 'SpectreOS Bhoo UI sync', urgency: 1 },
      { type: 'wait', ms: 60 },
    ],
  },
];

function formatBytes(n) {
  if (!Number.isFinite(n) || n <= 0) return '0 B';
  if (n >= 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(2)} MB`;
  if (n >= 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${n} B`;
}

function escapeHtml(s) {
  return String(s ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

function waitFrame() {
  return new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
}

function showLoader({
  title,
  sub = '',
  dlPct = null,
  dlText = '',
  dlIndeterminate = false,
  loadPct = null,
  loadText = '',
  loadIndeterminate = false,
  state = 'loading',
}) {
  if (loaderHideTimer) {
    clearTimeout(loaderHideTimer);
    loaderHideTimer = 0;
  }
  elStageLoader.hidden = false;
  elStageLoader.classList.remove('fade-out');
  elLoaderTitle.textContent = title;
  elLoaderSub.textContent = sub;
  elLoaderSpinner.classList.toggle('done', state === 'done');
  elLoaderSpinner.classList.toggle('error', state === 'error');
  elBtnLoaderRetry.hidden = state !== 'error';

  elDlLabel.textContent = dlText;
  elDlBarFill.classList.toggle('indeterminate', dlIndeterminate);
  elDlBarFill.classList.toggle('done', dlPct === 100 && state !== 'error');
  elDlBarFill.classList.toggle('error', state === 'error' && dlPct !== 100);
  if (!dlIndeterminate && dlPct !== null) {
    elDlBarFill.style.width = `${Math.max(0, Math.min(100, dlPct))}%`;
  }

  elLoadLabel.textContent = loadText;
  elLoadBarFill.classList.toggle('indeterminate', loadIndeterminate);
  elLoadBarFill.classList.toggle('done', loadPct === 100 && state !== 'error');
  elLoadBarFill.classList.toggle('error', state === 'error' && dlPct === 100);
  if (!loadIndeterminate && loadPct !== null) {
    elLoadBarFill.style.width = `${Math.max(0, Math.min(100, loadPct))}%`;
  }
}

function hideLoaderSoon() {
  if (elStageLoader.hidden) return;
  elStageLoader.classList.add('fade-out');
  if (loaderHideTimer) clearTimeout(loaderHideTimer);
  loaderHideTimer = setTimeout(() => {
    elStageLoader.hidden = true;
    elStageLoader.classList.remove('fade-out');
  }, 190);
}

function readUtf8(ptr, len) {
  if (!engine || !ptr || !len) return '';
  return decoder.decode(new Uint8Array(engine.memory.buffer, ptr, len));
}

async function preloadPersianFonts() {
  if (!document.fonts || !document.fonts.load) return;
  try {
    const loaded = await Promise.all([
      document.fonts.load('300 18px "Shabnam"', 'فارسی'),
      document.fonts.load('400 18px "Shabnam"', 'فارسی'),
      document.fonts.load('500 18px "Shabnam"', 'فارسی'),
      document.fonts.load('700 18px "Shabnam"', 'فارسی'),
    ]);
    const count = loaded.reduce((acc, list) => acc + list.length, 0);
    if (count > 0) {
      appendLog(`Fonts: Persian=Shabnam (${count} face weights ready)`);
    }
    request();
  } catch {}
}

if (document.fonts && document.fonts.ready) {
  document.fonts.ready.then(() => request());
}

function calibrateFonts() {
  ctx.save();
  for (let code = 32; code < 127; code++) {
    const ch = String.fromCharCode(code);
    ctx.font = '400 100px Arial, sans-serif';
    const w400 = ctx.measureText(ch).width / 100;
    ctx.font = '500 100px Arial, sans-serif';
    const w500 = ctx.measureText(ch).width / 100;
    ctx.font = '700 100px Arial, sans-serif';
    const w700 = ctx.measureText(ch).width / 100;
    engine.debug_font_advance(code, w400, Math.max(w500, w700));
  }
  const baselines = [400, 700].map(weight => {
    ctx.font = `${weight} 100px Arial, sans-serif`;
    const m = ctx.measureText('Hg');
    return ((m.fontBoundingBoxAscent ?? 80) - (m.fontBoundingBoxDescent ?? 20)) / 200;
  });
  ctx.restore();
  engine.debug_font_baseline(baselines[0], baselines[1]);
}

function syncClock() {
  if (!engine) return;
  const now = new Date();
  const unixSecs = now.getTime() / 1000;
  const tzOffsetSecs = -now.getTimezoneOffset() * 60;
  engine.debug_set_clock(unixSecs, tzOffsetSecs);
}

function updateViewport(w, h) {
  logicalW = Math.max(240, Math.min(1920, Math.round(w)));
  logicalH = Math.max(240, Math.min(1920, Math.round(h)));
  appCanvas.style.width = `${logicalW}px`;
  appCanvas.style.height = `${logicalH}px`;
  overlayCanvas.style.width = `${logicalW}px`;
  overlayCanvas.style.height = `${logicalH}px`;
  feedbackCanvas.style.width = `${logicalW}px`;
  feedbackCanvas.style.height = `${logicalH}px`;
  elCustomW.value = String(logicalW);
  elCustomH.value = String(logicalH);
  elMetaDims.textContent = `${logicalW} × ${logicalH} lp`;
  renderFeedbackCanvas();
  request();
}

function appendLog(line) {
  const row = document.createElement('div');
  row.className = 'log-entry';
  const ts = new Date().toLocaleTimeString('en-GB', { hour12: false });
  row.innerHTML = `<span class="ts">${ts}</span>`;
  row.append(document.createTextNode(line));
  elActionLog.prepend(row);
  while (elActionLog.children.length > 140) {
    elActionLog.lastElementChild.remove();
  }
}

// =============================================================================
// System Notification Banner Overlay & D-Bus Monitor
// =============================================================================

function injectDbusPayload(obj) {
  if (!engine || !engine.debug_inject_buffer || !engine.debug_inject_dbus) return false;
  const bytes = encoder.encode(JSON.stringify(obj)).subarray(0, 2048);
  const ptr = engine.debug_inject_buffer();
  new Uint8Array(engine.memory.buffer, ptr, bytes.length).set(bytes);
  engine.debug_inject_dbus(bytes.length);
  request();
  return true;
}

function dismissOverlayNotif(id, reason = 2) {
  const rec = activeOverlayNotifs.get(id);
  if (!rec) return;
  if (rec.timer) clearTimeout(rec.timer);
  rec.el.remove();
  activeOverlayNotifs.delete(id);
  recordBusEvent({
    dir: 'out',
    kind: 'dbus-signal',
    bus: 'session',
    service: 'org.freedesktop.Notifications',
    path: '/org/freedesktop/Notifications',
    interface: 'org.freedesktop.Notifications',
    member: 'NotificationClosed',
    args: { id, reason },
  });
}

function showOverlayNotification(notif) {
  if (!notif) return;
  // Wisp Notifications app already paints its own native canvas banner.
  if (apps[activeAppIdx] === 'notifications') return;
  const id = notif.id || Math.floor(Math.random() * 10000);
  if (activeOverlayNotifs.has(id)) dismissOverlayNotif(id, 3);

  const card = document.createElement('div');
  card.className = `sys-notif-card${notif.urgency >= 2 ? ' critical' : ''}`;
  const iconName = notif.urgency >= 2 ? 'exclamation-triangle' : 'bell-alert';
  const actionsHtml = (notif.actions || [])
    .map(label => `<button type="button" data-action="${escapeHtml(label)}">${escapeHtml(label)}</button>`)
    .join('');

  card.innerHTML = `
    <div class="sys-notif-top">
      <span class="sys-notif-app">${heroicon(iconName)}<span>${escapeHtml(notif.app || 'System')} · #${id}</span></span>
      <button type="button" class="sys-notif-dismiss" title="Dismiss notification">${heroicon('x-mark')}</button>
    </div>
    <div class="sys-notif-title">${escapeHtml(notif.title || '')}</div>
    ${notif.body ? `<div class="sys-notif-body">${escapeHtml(notif.body)}</div>` : ''}
    ${actionsHtml ? `<div class="sys-notif-actions">${actionsHtml}</div>` : ''}
  `;

  card.querySelector('.sys-notif-dismiss').addEventListener('click', () => {
    dismissOverlayNotif(id, 2);
  });
  for (const btn of card.querySelectorAll('.sys-notif-actions button')) {
    btn.addEventListener('click', () => {
      const actionKey = btn.dataset.action.toLowerCase().replace(/\s+/g, '_');
      recordBusEvent({
        dir: 'out',
        kind: 'dbus-signal',
        bus: 'session',
        service: 'org.freedesktop.Notifications',
        path: '/org/freedesktop/Notifications',
        interface: 'org.freedesktop.Notifications',
        member: 'ActionInvoked',
        args: { id, action_key: actionKey },
      });
      appendLog(`Notification #${id} action -> ${actionKey}`);
      dismissOverlayNotif(id, 2);
    });
  }

  elSysNotifStack.prepend(card);
  while (elSysNotifStack.children.length > 3) {
    elSysNotifStack.lastElementChild.remove();
  }
  const timer = notif.urgency >= 2 ? 0 : setTimeout(() => dismissOverlayNotif(id, 1), 6500);
  activeOverlayNotifs.set(id, { el: card, timer });
}

function recordBusEvent(ev) {
  const entry = {
    ts: new Date().toLocaleTimeString('en-GB', { hour12: false }),
    ...ev,
  };
  busEvents.unshift(entry);
  if (busEvents.length > 150) busEvents.length = 150;
  if (ev.notification) {
    showOverlayNotification(ev.notification);
  }
  if (activeTab !== 'sysbus') {
    unreadBusCount = Math.min(99, unreadBusCount + 1);
    elBusBadge.textContent = String(unreadBusCount);
    elBusBadge.hidden = false;
  }
  renderBusList();
}

function renderBusList() {
  const kindFilter = elBusFilterKind.value;
  elBusList.innerHTML = '';
  const items = busEvents.filter(e => kindFilter === 'all' || e.kind === kindFilter);
  if (!items.length) {
    elBusList.innerHTML = '<div class="testbar-empty">No D-Bus or system notification traffic captured yet. Interact with the app or click Send above.</div>';
    return;
  }
  for (const e of items) {
    const row = document.createElement('div');
    row.className = `bus-entry ${e.kind || 'dbus-call'}`;
    const dirLabel = e.dir === 'in' ? 'IN' : 'OUT';
    const kindLabel = e.kind === 'notification' ? 'NOTIFY' : e.kind === 'dbus-signal' ? 'SIGNAL' : 'CALL';
    const argsStr = typeof e.args === 'string' ? e.args : JSON.stringify(e.args);
    row.innerHTML = `
      <div class="bus-entry-head">
        <span class="bus-pill ${e.kind}">${kindLabel}</span>
        <span class="bus-pill ${e.dir === 'in' ? 'dir-in' : ''}">${dirLabel} · ${escapeHtml(e.bus || 'session')}</span>
        <span style="color:var(--muted);margin-left:auto">${e.ts}</span>
      </div>
      <div class="bus-member">${escapeHtml(e.interface)}.${escapeHtml(e.member)}</div>
      <div style="color:var(--muted);font-size:10px">${escapeHtml(e.service)} ${escapeHtml(e.path || '')}</div>
      <div class="bus-args">${escapeHtml(argsStr)}</div>
    `;
    elBusList.append(row);
  }
}

function sendSelectedBusPreset() {
  const preset = elBusPreset.value;
  const app = elBusNotifApp.value.trim() || 'SpectreOS';
  const title = elBusNotifTitle.value.trim() || 'Debug Notification';
  const body = elBusNotifBody.value.trim() || 'Injected from Sys & D-Bus monitor';
  switch (preset) {
    case 'notify':
      injectDbusPayload({ service: 'org.freedesktop.Notifications', interface: 'org.freedesktop.Notifications', member: 'Notify', app, title, body, urgency: 1 });
      break;
    case 'notify-critical':
      injectDbusPayload({ service: 'org.freedesktop.Notifications', interface: 'org.freedesktop.Notifications', member: 'Notify', app, title, body, urgency: 2 });
      break;
    case 'osk-fa':
      injectDbusPayload({ service: 'org.spectreos.OSK0', interface: 'org.spectreos.OSK0', member: 'SetLayout', layout: 'ir' });
      break;
    case 'osk-us':
      injectDbusPayload({ service: 'org.spectreos.OSK0', interface: 'org.spectreos.OSK0', member: 'SetLayout', layout: 'us' });
      break;
    case 'osk-term':
      injectDbusPayload({ service: 'org.spectreos.OSK0', interface: 'org.spectreos.OSK0', member: 'SetLayout', layout: 'terminal' });
      break;
    case 'portal-open':
      injectDbusPayload({ service: 'org.freedesktop.portal.Desktop', interface: 'org.freedesktop.portal.FileChooser', member: 'OpenFile', mode: 1 });
      break;
    case 'portal-save':
      injectDbusPayload({ service: 'org.freedesktop.portal.Desktop', interface: 'org.freedesktop.portal.FileChooser', member: 'SaveFile', value: body || 'export.txt' });
      break;
    case 'nm-wifi-on':
      injectDbusPayload({ service: 'org.freedesktop.NetworkManager', interface: 'org.freedesktop.NetworkManager', member: 'SetWifi', value: 'true' });
      break;
    case 'nm-wifi-off':
      injectDbusPayload({ service: 'org.freedesktop.NetworkManager', interface: 'org.freedesktop.NetworkManager', member: 'SetWifi', value: 'false' });
      break;
  }
}

elBusPreset.addEventListener('change', () => {
  const isNotif = elBusPreset.value.startsWith('notify');
  elBusNotifInputs.style.display = isNotif ? 'flex' : 'none';
  if (elBusPreset.value === 'portal-save') {
    elBusNotifBody.placeholder = 'Suggested filename (e.g. notes.txt)';
    elBusNotifBody.value = 'export.txt';
  } else if (isNotif) {
    elBusNotifBody.placeholder = 'Notification body';
  }
});
elBtnSendBus.addEventListener('click', sendSelectedBusPreset);
elBusFilterKind.addEventListener('change', renderBusList);
elBtnClearBus.addEventListener('click', () => {
  busEvents.length = 0;
  unreadBusCount = 0;
  elBusBadge.hidden = true;
  renderBusList();
});

// =============================================================================
// Input & State Test Cases — Button Creator, Recorder & Runner
// =============================================================================

function loadStoredTests() {
  let saved = [];
  try {
    const raw = localStorage.getItem(TESTS_STORAGE_KEY);
    if (raw) saved = JSON.parse(raw);
  } catch {}
  if (!Array.isArray(saved) || !saved.length) {
    testCases = BUILTIN_TESTS.map(t => structuredClone(t));
  } else {
    testCases = saved;
  }
}

async function syncRepoTests(overwrite = false) {
  try {
    const resp = await fetch('/__debug/tests', { cache: 'no-store' });
    if (!resp.ok) return;
    const repoTests = await resp.json();
    if (!Array.isArray(repoTests) || !repoTests.length) return;
    if (overwrite) {
      testCases = repoTests;
      saveTests();
      renderTestUI();
      appendLog(`Restored ${repoTests.length} test case(s) from test/web-tests.json`);
      return;
    }
    let changed = false;
    const repoApps = new Set(repoTests.map(t => t.app));
    const deduped = testCases.filter(t => !(t.builtin && repoApps.has(t.app)));
    if (deduped.length !== testCases.length) {
      testCases = deduped;
      changed = true;
    }
    for (const tc of repoTests) {
      if (!testCases.some(existing => existing.id === tc.id)) {
        testCases.push(tc);
        changed = true;
      }
    }
    if (changed) {
      saveTests();
      renderTestUI();
    }
  } catch {}
}

function saveTests() {
  try {
    localStorage.setItem(TESTS_STORAGE_KEY, JSON.stringify(testCases));
  } catch {}
}

function currentAppName() {
  return apps[activeAppIdx] || 'files';
}

function testsForCurrentApp() {
  const cur = currentAppName();
  return testCases.filter(t => t.app === '*' || t.app === cur);
}

function renderTestUI() {
  const curTests = testsForCurrentApp();

  // 1. Pinned Quick Test Button Bar
  elQuickTestPills.innerHTML = '';
  const pinned = curTests.filter(t => t.pinned);
  if (!pinned.length) {
    elQuickTestPills.innerHTML = '<span class="testbar-empty">No pinned test buttons for this app — click New Test or Record to create one.</span>';
  } else {
    for (const tc of pinned) {
      const res = testResults.get(tc.id);
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = `test-pill${res ? ' ' + res.status : ''}`;
      btn.title = res ? `${tc.name}: ${res.detail}` : `Run test case "${tc.name}" (${tc.steps.length} steps)`;
      const ico = res?.status === 'pass' ? 'check-circle' : res?.status === 'fail' ? 'x-circle' : 'play';
      btn.innerHTML = `${heroicon(ico)}<span>${escapeHtml(tc.name)}</span>`;
      btn.addEventListener('click', () => runTestCase(tc));
      elQuickTestPills.append(btn);
    }
  }

  // 2. Full Test Cases Inspector List
  elTestCaseList.innerHTML = '';
  if (!curTests.length) {
    elTestCaseList.innerHTML = '<div class="testbar-empty">No test cases for this app yet. Click New or Rec above to create a test button.</div>';
    return;
  }
  for (const tc of curTests) {
    const res = testResults.get(tc.id);
    const card = document.createElement('div');
    card.className = `test-card${res ? ' ' + res.status : ''}`;
    const statusTag = res && (res.status === 'pass' || res.status === 'fail')
      ? `<span class="status-tag ${res.status}">${res.status}</span>`
      : '';
    card.innerHTML = `
      <div class="test-card-top">
        <div class="test-card-title">
          ${heroicon(tc.pinned ? 'bookmark' : 'beaker')}
          <span class="name">${escapeHtml(tc.name)}</span>
          ${statusTag}
        </div>
        <div class="btn-row">
          <button type="button" class="mini icon-btn primary" data-act="run" title="Run test">${heroicon('play')}</button>
          <button type="button" class="mini icon-btn" data-act="pin" title="${tc.pinned ? 'Unpin from bar' : 'Pin to Test Bar'}">${heroicon('bookmark')}</button>
          <button type="button" class="mini icon-btn" data-act="edit" title="Edit test">${heroicon('pencil-square')}</button>
          <button type="button" class="mini icon-btn" data-act="del" title="Delete test">${heroicon('trash')}</button>
        </div>
      </div>
      <div class="test-card-meta">
        <span>${tc.steps.length} step(s)${tc.resetBefore ? ' · reset' : ''}</span>
        <span>${escapeHtml(res?.detail || (tc.app === '*' ? 'any app' : tc.app))}</span>
      </div>
    `;
    card.querySelector('[data-act="run"]').addEventListener('click', () => runTestCase(tc));
    card.querySelector('[data-act="pin"]').addEventListener('click', () => {
      tc.pinned = !tc.pinned;
      saveTests();
      renderTestUI();
    });
    card.querySelector('[data-act="edit"]').addEventListener('click', () => openTestEditor(tc));
    card.querySelector('[data-act="del"]').addEventListener('click', () => {
      testCases = testCases.filter(x => x.id !== tc.id);
      testResults.delete(tc.id);
      saveTests();
      renderTestUI();
    });
    elTestCaseList.append(card);
  }
}

function defaultStepForType(type) {
  switch (type) {
    case 'tapNode': return { type: 'tapNode', label: 'Images' };
    case 'tap': return { type: 'tap', x: 200, y: 200 };
    case 'type': return { type: 'type', text: 'hello' };
    case 'key': return { type: 'key', key: 'Enter' };
    case 'viewport': return { type: 'viewport', w: 832, h: 400 };
    case 'theme': return { type: 'theme', dark: true };
    case 'filesMode': return { type: 'filesMode', value: '2' };
    case 'notifMode': return { type: 'notifMode', value: '1' };
    case 'kbLayout': return { type: 'kbLayout', value: 'ir' };
    case 'pushNotif': return { type: 'pushNotif', app: 'SpectreOS', title: 'Test Alert', body: 'Triggered by test case', urgency: 1 };
    case 'dbus': return { type: 'dbus', preset: 'nm-wifi-on', value: 'true' };
    case 'back': return { type: 'back' };
    case 'wait': return { type: 'wait', ms: 100 };
    case 'assertState': return { type: 'assertState', path: 'darkTheme', expected: 'true' };
    default: return { type: 'wait', ms: 50 };
  }
}

function renderEditorSteps() {
  elTcStepList.innerHTML = '';
  editorSteps.forEach((step, idx) => {
    const row = document.createElement('div');
    row.className = 'step-row';
    let fieldsHtml = '';
    if (step.type === 'tapNode') {
      fieldsHtml = `<input data-k="label" value="${escapeHtml(step.label || '')}" placeholder="Node label substring">`;
    } else if (step.type === 'tap') {
      fieldsHtml = `<input data-k="x" type="number" value="${step.x ?? 0}" style="max-width:64px" placeholder="x"><input data-k="y" type="number" value="${step.y ?? 0}" style="max-width:64px" placeholder="y">`;
    } else if (step.type === 'type') {
      fieldsHtml = `<input data-k="text" value="${escapeHtml(step.text || '')}" placeholder="Text to type">`;
    } else if (step.type === 'key') {
      fieldsHtml = `<input data-k="key" value="${escapeHtml(step.key || 'Enter')}" placeholder="Enter / Escape / Tab">`;
    } else if (step.type === 'viewport') {
      fieldsHtml = `<input data-k="w" type="number" value="${step.w ?? 400}" style="max-width:68px"><input data-k="h" type="number" value="${step.h ?? 832}" style="max-width:68px">`;
    } else if (step.type === 'theme') {
      fieldsHtml = `<select data-k="dark" class="mini-select"><option value="true" ${step.dark ? 'selected' : ''}>Dark</option><option value="false" ${!step.dark ? 'selected' : ''}>Light</option></select>`;
    } else if (step.type === 'filesMode' || step.type === 'notifMode' || step.type === 'kbLayout') {
      fieldsHtml = `<input data-k="value" value="${escapeHtml(step.value ?? '0')}" placeholder="value (e.g. 0..4 or ir/us/terminal)">`;
    } else if (step.type === 'pushNotif') {
      fieldsHtml = `<input data-k="title" value="${escapeHtml(step.title || '')}" placeholder="Title"><input data-k="body" value="${escapeHtml(step.body || '')}" placeholder="Body">`;
    } else if (step.type === 'dbus') {
      fieldsHtml = `<input data-k="preset" value="${escapeHtml(step.preset || 'notify')}" placeholder="preset"><input data-k="value" value="${escapeHtml(step.value || '')}" placeholder="value">`;
    } else if (step.type === 'wait') {
      fieldsHtml = `<input data-k="ms" type="number" value="${step.ms ?? 100}" style="max-width:74px"><span>ms</span>`;
    } else if (step.type === 'assertState') {
      fieldsHtml = `<input data-k="path" value="${escapeHtml(step.path || '')}" placeholder="state key (e.g. layout)"><input data-k="expected" value="${escapeHtml(step.expected || '')}" placeholder="value or >=1">`;
    } else {
      fieldsHtml = '<span style="color:var(--muted)">Trigger app back</span>';
    }
    row.innerHTML = `
      <span class="step-kind">${escapeHtml(step.type)}</span>
      ${fieldsHtml}
      <button type="button" class="mini icon-btn" data-rm="${idx}" title="Remove step">${heroicon('x-mark')}</button>
    `;
    for (const inp of row.querySelectorAll('[data-k]')) {
      const handler = () => {
        const k = inp.dataset.k;
        if (k === 'dark') step[k] = inp.value === 'true';
        else if (inp.type === 'number') step[k] = Number(inp.value);
        else step[k] = inp.value;
      };
      inp.addEventListener('input', handler);
      inp.addEventListener('change', handler);
    }
    row.querySelector('[data-rm]').addEventListener('click', () => {
      editorSteps.splice(idx, 1);
      renderEditorSteps();
    });
    elTcStepList.append(row);
  });
}

function switchTab(tabName) {
  activeTab = tabName;
  for (const b of document.querySelectorAll('.tab')) {
    const selected = b.dataset.tab === tabName;
    b.classList.toggle('active', selected);
    b.setAttribute('aria-selected', String(selected));
    b.tabIndex = selected ? 0 : -1;
  }
  document.getElementById('tabState').hidden = tabName !== 'state';
  document.getElementById('tabTests').hidden = tabName !== 'tests';
  document.getElementById('tabSysBus').hidden = tabName !== 'sysbus';
  document.getElementById('tabSemantics').hidden = tabName !== 'semantics';
  if (tabName === 'semantics') refreshSemanticsTree();
  if (tabName === 'sysbus') {
    unreadBusCount = 0;
    elBusBadge.hidden = true;
    renderBusList();
  }
  if (tabName === 'tests') renderTestUI();
}

function openTestEditor(existing = null, prefillSteps = null) {
  elChkModalInspector.checked = true;
  elInspectorPane.hidden = false;
  elInspectorSplitter.hidden = false;
  switchTab('tests');
  editingTestId = existing ? existing.id : null;
  elTestEditorTitle.textContent = existing ? 'Edit Test Case Button' : 'New Test Case Button';
  elTcName.value = existing ? existing.name : '';
  elTcApp.value = existing ? existing.app : currentAppName();
  elTcPin.checked = existing ? Boolean(existing.pinned) : true;
  elTcReset.checked = existing ? Boolean(existing.resetBefore) : false;
  editorSteps = existing
    ? structuredClone(existing.steps)
    : prefillSteps
      ? structuredClone(prefillSteps)
      : [defaultStepForType('assertState')];
  renderEditorSteps();
  elTestEditorForm.hidden = false;
  elTcName.focus();
}

function closeTestEditor() {
  elTestEditorForm.hidden = true;
  editingTestId = null;
}

function toggleRecording() {
  if (!isRecording) {
    isRecording = true;
    recordedSteps = [];
    elBtnBarRecord.classList.add('recording');
    elBtnRecordTest.classList.add('recording');
    elBarRecordLabel.textContent = 'Stop Rec';
    elTabRecordLabel.textContent = 'Stop';
    appendLog('Recording live inputs… click Stop when done to save as a button.');
  } else {
    isRecording = false;
    elBtnBarRecord.classList.remove('recording');
    elBtnRecordTest.classList.remove('recording');
    elBarRecordLabel.textContent = 'Record';
    elTabRecordLabel.textContent = 'Rec';
    if (recordedSteps.length) {
      appendLog(`Recorded ${recordedSteps.length} step(s) — name & save your test button.`);
      openTestEditor(null, recordedSteps);
    } else {
      appendLog('Recording stopped (no inputs recorded).');
    }
  }
}

function getStatePath(obj, path) {
  return String(path)
    .split('.')
    .reduce((acc, k) => (acc !== null && acc !== undefined ? acc[k] : undefined), obj);
}

function checkAssertion(actual, expected) {
  const exp = String(expected ?? '').trim();
  if (exp.startsWith('>=')) return Number(actual) >= Number(exp.slice(2));
  if (exp.startsWith('<=')) return Number(actual) <= Number(exp.slice(2));
  if (exp.startsWith('>')) return Number(actual) > Number(exp.slice(1));
  if (exp.startsWith('<')) return Number(actual) < Number(exp.slice(1));
  if (exp.startsWith('~')) return String(actual ?? '').toLowerCase().includes(exp.slice(1).toLowerCase());
  return String(actual) === exp;
}

async function simulateTap(x, y) {
  pointer = [x, y];
  const pointerBit = elChkTouch.checked ? 128 : 0;
  push(pointerBit | 1);
  await waitFrame();
  push(pointerBit | 2);
  await waitFrame();
}

async function runTestCase(tc) {
  if (!engine) return false;
  testResults.set(tc.id, { status: 'running', detail: 'Running…' });
  renderTestUI();
  appendLog(`Running test "${tc.name}"…`);
  try {
    if (tc.app && tc.app !== '*' && apps[activeAppIdx] !== tc.app) {
      const idx = apps.indexOf(tc.app);
      if (idx >= 0) {
        selectApp(idx);
        await waitFrame();
      }
    }
    if (tc.resetBefore) {
      closeEditor();
      engine.debug_reset();
      request();
      await waitFrame();
    }

    for (const step of tc.steps) {
      switch (step.type) {
        case 'viewport':
          updateViewport(Number(step.w) || 400, Number(step.h) || 832);
          await waitFrame();
          break;
        case 'theme':
          elChkDark.checked = Boolean(step.dark);
          request();
          await waitFrame();
          break;
        case 'filesMode':
          if (engine.debug_files_mode) {
            elFilesMode.value = String(step.value ?? '0');
            engine.debug_files_mode(Number(step.value) || 0);
            request();
            await waitFrame();
          }
          break;
        case 'notifMode':
          if (engine.debug_notif_mode) {
            elNotifMode.value = String(step.value ?? '0');
            engine.debug_notif_mode(Number(step.value) || 0);
            request();
            await waitFrame();
          }
          break;
        case 'kbLayout':
          if (engine.debug_keyboard_layout) {
            const code = step.value === 'ir' || step.value === '1' ? 1 : step.value === 'terminal' || step.value === '2' ? 2 : 0;
            engine.debug_keyboard_layout(code);
            request();
            await waitFrame();
          }
          break;
        case 'pushNotif':
          injectDbusPayload({
            service: 'org.freedesktop.Notifications',
            interface: 'org.freedesktop.Notifications',
            member: 'Notify',
            app: step.app || 'Test',
            title: step.title || 'Test Alert',
            body: step.body || '',
            urgency: Number(step.urgency ?? 1),
          });
          await waitFrame();
          break;
        case 'dbus': {
          const prev = elBusPreset.value;
          if ([...elBusPreset.options].some(o => o.value === step.preset)) {
            elBusPreset.value = step.preset;
            if (step.value) elBusNotifBody.value = String(step.value);
            sendSelectedBusPreset();
            elBusPreset.value = prev;
          } else {
            injectDbusPayload({
              service: step.service || 'org.spectreos.Debug',
              interface: step.interface || 'org.spectreos.Debug',
              member: step.preset || 'Invoke',
              value: String(step.value ?? ''),
            });
          }
          await waitFrame();
          break;
        }
        case 'tap':
          await simulateTap(Number(step.x) || 0, Number(step.y) || 0);
          break;
        case 'tapNode': {
          const q = String(step.label || '').toLowerCase();
          const hit = nodes.find(n => (n.label || '').toLowerCase().includes(q));
          if (!hit) throw new Error(`Semantic node "${step.label}" not found`);
          await simulateTap(hit.bounds[0] + hit.bounds[2] * 0.5, hit.bounds[1] + hit.bounds[3] * 0.5);
          break;
        }
        case 'type': {
          const txt = String(step.text || '');
          if (editor && activeFieldId) {
            editor.value = txt;
            editor.dispatchEvent(new Event('input'));
          } else if (nodes.some(n => n.role === 'text field')) {
            const tf = nodes.find(n => n.role === 'text field');
            openEditor(tf.id, tf.label);
            if (editor) {
              editor.value = txt;
              editor.dispatchEvent(new Event('input'));
            }
          }
          await waitFrame();
          break;
        }
        case 'key': {
          const map = { Tab: 1, Enter: 2, ' ': 3, ArrowLeft: 4, ArrowRight: 5, ArrowUp: 6, ArrowDown: 7, Home: 8, End: 9, Escape: 10 };
          const code = map[step.key] || 2;
          push(16, code);
          await waitFrame();
          break;
        }
        case 'back':
          engine.debug_back();
          request();
          await waitFrame();
          break;
        case 'wait':
          await sleep(Math.min(5000, Math.max(0, Number(step.ms) || 50)));
          await waitFrame();
          break;
        case 'assertState': {
          await waitFrame();
          const actual = getStatePath(lastParsedState, step.path);
          if (!checkAssertion(actual, step.expected)) {
            throw new Error(`Expected ${step.path} ${step.expected}, got ${JSON.stringify(actual)}`);
          }
          break;
        }
      }
    }

    testResults.set(tc.id, { status: 'pass', detail: `Passed (${tc.steps.length} steps)` });
    appendLog(`PASS: ${tc.name}`);
    renderTestUI();
    return true;
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    testResults.set(tc.id, { status: 'fail', detail: msg });
    appendLog(`FAIL: ${tc.name} — ${msg}`);
    renderTestUI();
    return false;
  }
}

async function runAllCurrentTests() {
  if (runningTests) return;
  runningTests = true;
  let passed = 0;
  const list = testsForCurrentApp();
  for (const tc of list) {
    if (await runTestCase(tc)) passed++;
  }
  appendLog(`Suite complete: ${passed}/${list.length} passed`);
  runningTests = false;
}

elBtnNewTest.addEventListener('click', () => openTestEditor());
elBtnBarNewTest.addEventListener('click', () => openTestEditor());
elBtnCloseTestEditor.addEventListener('click', closeTestEditor);
elBtnCancelTest.addEventListener('click', closeTestEditor);
elBtnRecordTest.addEventListener('click', toggleRecording);
elBtnBarRecord.addEventListener('click', toggleRecording);
elBtnRunAllTests.addEventListener('click', runAllCurrentTests);
elBtnBarRunAll.addEventListener('click', runAllCurrentTests);
elBtnAddStep.addEventListener('click', () => {
  editorSteps.push(defaultStepForType(elTcAddStepType.value));
  renderEditorSteps();
});

elTestEditorForm.addEventListener('submit', e => {
  e.preventDefault();
  const name = elTcName.value.trim();
  if (!name) return;
  const item = {
    id: editingTestId || `tc-${Date.now()}`,
    name,
    app: elTcApp.value,
    pinned: elTcPin.checked,
    resetBefore: elTcReset.checked,
    steps: structuredClone(editorSteps),
  };
  if (editingTestId) {
    const idx = testCases.findIndex(t => t.id === editingTestId);
    if (idx >= 0) testCases[idx] = item;
  } else {
    testCases.unshift(item);
  }
  saveTests();
  closeTestEditor();
  renderTestUI();
  appendLog(`Saved test button "${name}"`);
});

elBtnExportTests.addEventListener('click', () => {
  const blob = new Blob([JSON.stringify(testCases, null, 2)], { type: 'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = 'bhoo-debug-tests.json';
  a.click();
  URL.revokeObjectURL(a.href);
});

elBtnImportTests.addEventListener('click', () => elImportTestsFile.click());
elImportTestsFile.addEventListener('change', async () => {
  const file = elImportTestsFile.files?.[0];
  if (!file) return;
  try {
    const parsed = JSON.parse(await file.text());
    if (!Array.isArray(parsed)) throw new Error('Expected JSON array of test cases');
    for (const tc of parsed) {
      if (tc && tc.name && Array.isArray(tc.steps)) {
        tc.id ||= `tc-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
        const existing = testCases.findIndex(x => x.id === tc.id);
        if (existing >= 0) testCases[existing] = tc;
        else testCases.unshift(tc);
      }
    }
    saveTests();
    renderTestUI();
    appendLog(`Imported test cases from ${file.name}`);
  } catch (err) {
    appendLog(`Import failed: ${err}`);
  }
  elImportTestsFile.value = '';
});

// =============================================================================
// Core Bhoo Frame Loop & Text Adapter
// =============================================================================

function closeEditor() {
  if (editor) {
    const old = editor;
    editor = null;
    activeFieldId = null;
    if (old.parentNode) old.remove();
    if (engine) engine.debug_adapter_focus(0n, 0);
  }
}

function openEditor(fieldIdStr, label) {
  if (!engine) return;
  if (activeFieldId === fieldIdStr && editor) return;
  closeEditor();
  const fieldId = BigInt(fieldIdStr);
  activeFieldId = fieldIdStr;
  editor = document.createElement('input');
  editor.type = 'text';
  editor.className = 'bhoo-edit';
  editor.setAttribute('aria-label', label || 'Text field');
  const len = engine.debug_text_len(fieldId);
  const ptr = engine.debug_text_ptr(fieldId);
  editor.value = readUtf8(ptr, len);
  editor.oninput = () => {
    const bytes = encoder.encode(editor.value).subarray(0, 512);
    const buf = engine.debug_edit_buffer();
    new Uint8Array(engine.memory.buffer, buf, bytes.length).set(bytes);
    engine.debug_edit_commit(fieldId, bytes.length);
    if (isRecording) {
      const last = recordedSteps.at(-1);
      if (last && last.type === 'type') last.text = editor.value;
      else recordedSteps.push({ type: 'type', text: editor.value });
    }
    request();
  };
  editor.onkeydown = e => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (isRecording) recordedSteps.push({ type: 'key', key: 'Enter' });
      push(16, 2);
      editor.blur();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      editor.blur();
    }
  };
  editor.onblur = () => {
    closeEditor();
    request();
  };
  document.body.append(editor);
  engine.debug_adapter_focus(fieldId, 1);
  editor.focus();
  request();
}

function request() {
  if (!engine || frameTrapped) return;
  if (!raf && !document.hidden) {
    raf = requestAnimationFrame(render);
  }
}

function push(flags = 0, key = 0, scroll = 0) {
  if (frameTrapped) return;
  const ev = { flags, key, scroll, x: pointer[0], y: pointer[1] };
  const last = queue.at(-1);
  if (
    last &&
    !(flags & 127) &&
    !key &&
    !(last.flags & 127) &&
    !last.key &&
    (flags & 512) === (last.flags & 512)
  ) {
    ev.scroll += last.scroll;
    queue[queue.length - 1] = ev;
  } else {
    queue.push(ev);
  }
  request();
}

function drawOverlay(dpr) {
  if (overlayCanvas.width !== Math.round(logicalW * dpr) || overlayCanvas.height !== Math.round(logicalH * dpr)) {
    overlayCanvas.width = Math.round(logicalW * dpr);
    overlayCanvas.height = Math.round(logicalH * dpr);
  }
  octx.setTransform(dpr, 0, 0, dpr, 0, 0);
  octx.clearRect(0, 0, logicalW, logicalH);

  const showAll = elChkBounds.checked;
  if (!showAll && !highlightNodeId) return;

  for (const n of nodes) {
    const isHi = highlightNodeId && n.id === highlightNodeId;
    if (!showAll && !isHi) continue;
    const [x, y, w, h] = n.bounds;
    if (w <= 0 || h <= 0) continue;
    octx.strokeStyle = isHi
      ? '#ff6b9d'
      : n.role === 'text field'
        ? 'rgba(91, 214, 160, 0.9)'
        : 'rgba(122, 162, 255, 0.55)';
    octx.lineWidth = isHi ? 2 : 1;
    octx.strokeRect(x + 0.5, y + 0.5, Math.max(0, w - 1), Math.max(0, h - 1));
    if (isHi) {
      octx.fillStyle = 'rgba(255, 107, 157, 0.14)';
      octx.fillRect(x, y, w, h);
      const tag = `${n.role} · ${n.label || '#' + n.id}`;
      octx.font = '600 10.5px ui-monospace, monospace';
      const tw = octx.measureText(tag).width + 8;
      const ty = Math.max(2, y - 16);
      octx.fillStyle = '#141824';
      octx.fillRect(x, ty, tw, 15);
      octx.fillStyle = '#ff98bb';
      octx.fillText(tag, x + 4, ty + 11);
    }
  }
}

function updateHoverInspector() {
  const [px, py] = pointer;
  if (!Number.isFinite(px) || !Number.isFinite(py)) return;
  let hit = null;
  for (let i = nodes.length - 1; i >= 0; i--) {
    const n = nodes[i];
    const [x, y, w, h] = n.bounds;
    if (px >= x && px < x + w && py >= y && py < y + h) {
      hit = n;
      break;
    }
  }
  if (hit) {
    const [x, y, w, h] = hit.bounds.map(v => Math.round(v));
    elHoverNode.textContent = JSON.stringify(
      {
        id: hit.id,
        role: hit.role,
        label: hit.label,
        bounds: [x, y, w, h],
        enabled: hit.enabled,
        focused: hit.focused,
        selected: hit.selected,
      },
      null,
      2
    );
    if (elChkBounds.checked) highlightNodeId = hit.id;
  } else {
    elHoverNode.textContent = `No semantic target at (${Math.round(px)}, ${Math.round(py)})`;
    if (elChkBounds.checked) highlightNodeId = null;
  }
}

function refreshSemanticsTree() {
  if (document.getElementById('tabSemantics').hidden) return;
  const q = elSemFilter.value.trim().toLowerCase();
  elSemList.innerHTML = '';
  for (const n of nodes) {
    const hay = `${n.role} ${n.label} ${n.id}`.toLowerCase();
    if (q && !hay.includes(q)) continue;
    const [x, y, w, h] = n.bounds.map(v => Math.round(v));
    const item = document.createElement('div');
    item.className = 'sem-item';
    item.innerHTML = `<span class="role">${escapeHtml(n.role)}</span> <span class="dims">${x},${y} ${w}×${h}</span><div>${escapeHtml(n.label || '(unlabeled)')}</div>`;
    item.onmouseenter = () => {
      highlightNodeId = n.id;
      drawOverlay(Math.min(devicePixelRatio || 1, 2));
    };
    item.onmouseleave = () => {
      highlightNodeId = null;
      drawOverlay(Math.min(devicePixelRatio || 1, 2));
    };
    elSemList.append(item);
  }
}

function render(time) {
  raf = 0;
  if (!engine) return;
  if (wakeTimer) {
    clearTimeout(wakeTimer);
    wakeTimer = 0;
  }
  try {
    const dpr = Math.min(devicePixelRatio || 1, 2);
    const pw = Math.round(logicalW * dpr);
    const ph = Math.round(logicalH * dpr);
    if (appCanvas.width !== pw || appCanvas.height !== ph) {
      appCanvas.width = pw;
      appCanvas.height = ph;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const ev = queue.shift() || { x: pointer[0], y: pointer[1], flags: 0, key: 0, scroll: 0 };
    const opts = (elChkDark.checked ? 1 : 0) | (elChkReduced.checked ? 2 : 0);

    const t0 = performance.now();
    const ptr = engine.debug_frame(
      logicalW,
      logicalH,
      time / 1000,
      ev.x,
      ev.y,
      ev.flags,
      ev.key,
      ev.scroll,
      opts
    );
    const packetLen = engine.debug_packet_len();
    const t1 = performance.now();

    const packet = new Uint8Array(engine.memory.buffer, ptr, packetLen);
    const frame = paintPacket(ctx, engine, packet, [0, 0, logicalW, logicalH], [0, 0, logicalW, logicalH]);
    const t2 = performance.now();

    if (!isLoadingWasm && !elStageLoader.hidden && elBtnLoaderRetry.hidden) {
      hideLoaderSoon();
    }

    nodes = frame.semantics;
    drawOverlay(dpr);

    const focusReq = engine.debug_take_focus_request();
    if (focusReq !== 0n) {
      if (focusReq === 1n) {
        closeEditor();
      } else {
        const idStr = focusReq.toString();
        const found = nodes.find(n => n.id === idStr);
        openEditor(idStr, found ? found.label : 'Edit');
      }
    }
    if (editor && activeFieldId) {
      const n = nodes.find(n => n.role === 'text field' && n.id === activeFieldId);
      if (!n) {
        closeEditor();
      } else {
        const fid = BigInt(activeFieldId);
        const cur = readUtf8(engine.debug_text_ptr(fid), engine.debug_text_len(fid));
        if (editor.value !== cur) editor.value = cur;
      }
    }

    // Drain Action logs from WASM.
    const logJson = readUtf8(engine.debug_take_logs_ptr(), engine.debug_take_logs_len());
    if (logJson && logJson !== '[]') {
      try {
        for (const msg of JSON.parse(logJson)) appendLog(msg);
      } catch {}
    }

    // Drain System Notification & D-Bus events from WASM.
    if (engine.debug_take_bus_ptr && engine.debug_take_bus_len) {
      const busRaw = readUtf8(engine.debug_take_bus_ptr(), engine.debug_take_bus_len());
      if (busRaw && busRaw !== '[]') {
        try {
          for (const bev of JSON.parse(busRaw)) recordBusEvent(bev);
        } catch {}
      }
    }

    // Update Live App State JSON.
    const stateRaw = readUtf8(engine.debug_state_ptr(), engine.debug_state_len());
    if (stateRaw !== lastStateHash) {
      lastStateHash = stateRaw;
      try {
        lastParsedState = JSON.parse(stateRaw);
        elAppStateJson.textContent = JSON.stringify(lastParsedState, null, 2);
      } catch {
        lastParsedState = {};
        elAppStateJson.textContent = stateRaw;
      }
    }

    const dt = lastFrameStamp ? time - lastFrameStamp : 0;
    lastFrameStamp = time;
    elMWasm.textContent = `${(t1 - t0).toFixed(2)} ms`;
    elMPaint.textContent = `${(t2 - t1).toFixed(2)} ms`;
    elMCmds.textContent = String(frame.commands);
    elMNodes.textContent = String(nodes.length);
    elMBytes.textContent = packetLen > 1024 ? `${(packetLen / 1024).toFixed(1)} KB` : `${packetLen} B`;
    elMLoop.textContent = frame.active ? 'active' : 'idle';
    elMetaFps.textContent = frame.active && dt > 0 ? `FPS: ${Math.round(1000 / dt)}` : 'FPS: idle';

    if (time - lastSemRender > 150) {
      lastSemRender = time;
      refreshSemanticsTree();
    }

    window.bhooDebug = {
      apps,
      activeApp: apps[activeAppIdx],
      nodes,
      text: frame.text,
      state: lastParsedState,
      busEvents,
      testCases,
      selectApp,
      injectDbus: injectDbusPayload,
      runTestCase,
      runAllTests: runAllCurrentTests,
      registerTestCase: tc => {
        if (!tc || !tc.name || !Array.isArray(tc.steps)) return;
        const item = {
          id: tc.id || `ext-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
          name: tc.name,
          app: tc.app || '*',
          pinned: tc.pinned ?? true,
          resetBefore: Boolean(tc.resetBefore),
          steps: tc.steps,
        };
        const idx = testCases.findIndex(x => x.id === item.id);
        if (idx >= 0) testCases[idx] = item;
        else testCases.unshift(item);
        saveTests();
        renderTestUI();
      },
      back: () => {
        if (engine.debug_back()) request();
      },
      reset: () => {
        closeEditor();
        engine.debug_reset();
        request();
      },
    };

    if (frame.active || queue.length) {
      request();
    } else if (frame.wake >= 0) {
      wakeTimer = setTimeout(request, Math.max(1, frame.wake * 1000 - time));
    }
  } catch (err) {
    if (err instanceof WebAssembly.RuntimeError) {
      frameTrapped = true;
      queue.length = 0;
      elMetaFps.textContent = 'FPS: stopped';
      appendLog(`ERROR: ${err} — use Reset to reload the preview`);
    } else {
      appendLog(`ERROR: ${err}`);
    }
    console.error(err);
  }
}

function selectApp(index) {
  if (!engine || index < 0 || index >= apps.length) return;
  closeEditor();
  activeAppIdx = index;
  engine.debug_select_app(index);
  updateAppPickerButton();
  if (!elAppPickerModal.hidden) renderAppCards();
  const name = apps[index];
  elFilesModeWrap.hidden = name !== 'files';
  elNotifControlsWrap.hidden = name !== 'notifications';
  if (name === 'files') elFilesMode.value = '0';
  appendLog(`Switched app -> ${name}`);
  const url = new URL(location.href);
  url.searchParams.set('app', name);
  history.replaceState(null, '', url);
  renderTestUI();
  request();
}

async function fetchWasmWithProgress(url, isReload) {
  const resp = await fetch(url, { cache: 'no-store' });
  if (!resp.ok) throw new Error(`HTTP ${resp.status} loading debug.wasm`);
  const total = Number(resp.headers.get('content-length')) || 0;
  const version = resp.headers.get('X-Debug-Wasm-Version');

  if (!resp.body || !resp.body.getReader) {
    showLoader({
      title: isReload ? 'Hot-reloading WebAssembly…' : 'Downloading WebAssembly…',
      sub: 'Fetching debug.wasm…',
      dlIndeterminate: true,
      dlText: total ? formatBytes(total) : 'Downloading…',
      loadPct: 0,
      loadText: 'Waiting…',
    });
    const buf = await resp.arrayBuffer();
    showLoader({
      title: isReload ? 'Hot-reloading WebAssembly…' : 'Downloading WebAssembly…',
      sub: `Downloaded ${formatBytes(buf.byteLength)}`,
      dlPct: 100,
      dlText: `${formatBytes(buf.byteLength)} · 100%`,
      loadIndeterminate: true,
      loadText: 'Compiling…',
    });
    return { bytes: buf, version };
  }

  const reader = resp.body.getReader();
  const chunks = [];
  let received = 0;
  let lastPaint = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    received += value.byteLength;
    const now = performance.now();
    if (now - lastPaint >= 32 || (total > 0 && received >= total)) {
      lastPaint = now;
      const pct = total > 0 ? Math.min(100, Math.round((received / total) * 100)) : null;
      const dlText = total > 0
        ? `${formatBytes(received)} / ${formatBytes(total)} (${pct}%)`
        : formatBytes(received);
      elBuildStatus.textContent = total > 0 ? `Downloading ${pct}%…` : `Downloading ${formatBytes(received)}…`;
      showLoader({
        title: isReload ? 'Hot-reloading WebAssembly…' : 'Downloading WebAssembly…',
        sub: 'Streaming debug.wasm from local server',
        dlPct: pct ?? 0,
        dlIndeterminate: total === 0,
        dlText,
        loadPct: 0,
        loadText: 'Waiting…',
      });
    }
  }

  const merged = new Uint8Array(received);
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return { bytes: merged.buffer, version };
}

function updateWasmReloadButton() {
  elBtnReloadWasm.hidden = !engine || !buildReady || availableVersion === null ||
    availableVersion === loadedVersion;
  elBtnReloadWasm.disabled = isLoadingWasm;
}

async function loadWasm(isReload = false) {
  if (isLoadingWasm) return;
  const requestedVersion = availableVersion;
  isLoadingWasm = true;
  updateWasmReloadButton();
  elBuildStatus.textContent = isReload ? 'Reloading WASM…' : 'Downloading WASM…';
  showLoader({
    title: isReload ? 'Hot-reloading WebAssembly…' : 'Downloading WebAssembly…',
    sub: 'Connecting to debug server…',
    dlPct: 0,
    dlText: '0%',
    loadPct: 0,
    loadText: 'Waiting…',
  });

  try {
    const { bytes, version } = await fetchWasmWithProgress(`/debug.wasm?t=${Date.now()}`, isReload);
    const sizeLabel = formatBytes(bytes.byteLength);

    elBuildStatus.textContent = 'Compiling WASM…';
    showLoader({
      title: 'Loading WebAssembly…',
      sub: 'Compiling module & initializing libbhoo',
      dlPct: 100,
      dlText: `${sizeLabel} · 100%`,
      loadIndeterminate: true,
      loadText: 'Compiling…',
    });
    await new Promise(r => requestAnimationFrame(r));

    const mod = await WebAssembly.compile(bytes);
    const imports = {};
    for (const imp of WebAssembly.Module.imports(mod)) {
      imports[imp.module] ||= {};
      if (imp.kind === 'function') imports[imp.module][imp.name] = () => 0;
    }

    showLoader({
      title: 'Starting Bhoo runtime…',
      sub: 'Instantiating WASM & calibrating font metrics',
      dlPct: 100,
      dlText: `${sizeLabel} · 100%`,
      loadPct: 80,
      loadText: '80% · Init',
    });
    const instance = await WebAssembly.instantiate(mod, imports);
    engine = instance.exports;
    frameTrapped = false;
    queue.length = 0;
    lastFrameStamp = 0;
    calibrateFonts();
    syncClock();

    const appsJson = readUtf8(engine.debug_list_apps_ptr(), engine.debug_list_apps_len());
    apps = JSON.parse(appsJson);
    updateAppPickerButton();
    if (!elAppPickerModal.hidden) renderAppCards();

    const wanted = new URLSearchParams(location.search).get('app');
    const idx = wanted ? Math.max(0, apps.indexOf(wanted.toLowerCase())) : Math.min(activeAppIdx, apps.length - 1);
    showLoader({
      title: 'Ready',
      sub: `Loaded ${apps.join(', ')}`,
      dlPct: 100,
      dlText: `${sizeLabel} · 100%`,
      loadPct: 100,
      loadText: '100% · Ready',
      state: 'done',
    });
    selectApp(idx);
    loadedVersion = version ?? requestedVersion;
    // The download may have already picked up a build that polling has yet to see.
    if (availableVersion === requestedVersion && version !== null) availableVersion = version;
    elBuildStatus.textContent = `Ready · ${(bytes.byteLength / 1024).toFixed(0)} KB WASM`;
    appendLog(isReload ? 'Hot-reloaded debug.wasm' : `Loaded debug.wasm (${apps.join(', ')})`);
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    elBuildStatus.textContent = 'Load failed';
    showLoader({
      title: 'Failed to load debug.wasm',
      sub: msg,
      dlPct: 0,
      dlText: 'Error',
      loadPct: 0,
      loadText: 'Failed',
      state: 'error',
    });
    appendLog(`ERROR: ${msg}`);
    console.error(err);
  } finally {
    isLoadingWasm = false;
    updateWasmReloadButton();
  }
}

async function pollVersion() {
  try {
    const response = await fetch('/__debug/version', { cache: 'no-store' });
    if (!response.ok) return;
    const info = await response.json();
    buildReady = info.status === 'ready';
    if (buildReady) {
      availableVersion = info.mtime == null ? null : String(info.mtime);
      // Older debug servers do not report the version in the download headers.
      if (engine && !isLoadingWasm && loadedVersion === null) loadedVersion = availableVersion;
    }
    if (info.watch && !isLoadingWasm) {
      elBuildStatus.textContent = `Watch · ${info.status || 'ready'}`;
    }
    updateWasmReloadButton();
  } catch {}
}

elBtnReloadWasm.addEventListener('click', () => loadWasm(true));
elBtnLoaderRetry.addEventListener('click', () => loadWasm(Boolean(engine)));

// Wire topbar controls
elBtnAppPicker.addEventListener('click', openAppPickerModal);
elBtnCloseAppPicker.addEventListener('click', closeAppPickerModal);
elAppPickerModal.addEventListener('click', e => {
  if (e.target === elAppPickerModal) closeAppPickerModal();
});
elAppPickerSearch.addEventListener('input', renderAppCards);
elAppPickerSearch.addEventListener('keydown', e => {
  if (e.key === 'Escape') {
    e.preventDefault();
    closeAppPickerModal();
  } else if (e.key === 'Enter') {
    const first = elAppCardGrid.querySelector('.app-card');
    if (first) {
      e.preventDefault();
      first.click();
    }
  }
});
elFilesMode.addEventListener('change', () => {
  if (!engine) return;
  closeEditor();
  engine.debug_files_mode(Number(elFilesMode.value));
  if (isRecording) recordedSteps.push({ type: 'filesMode', value: elFilesMode.value });
  appendLog(`Files mode -> ${elFilesMode.selectedOptions[0].textContent}`);
  request();
});

elNotifMode.addEventListener('change', () => {
  if (!engine || !engine.debug_notif_mode) return;
  closeEditor();
  engine.debug_notif_mode(Number(elNotifMode.value));
  if (isRecording) recordedSteps.push({ type: 'notifMode', value: elNotifMode.value });
  appendLog(`Notifications mode -> ${elNotifMode.selectedOptions[0].textContent}`);
  request();
});

function pushSampleNotif(kind, label) {
  if (!engine || !engine.debug_notif_push) return;
  engine.debug_notif_push(kind);
  appendLog(`Queued ${label} notification`);
  request();
}

elBtnNotifNormal.addEventListener('click', () => pushSampleNotif(0, 'normal'));
elBtnNotifActions.addEventListener('click', () => pushSampleNotif(1, 'interactive actions'));
elBtnNotifCritical.addEventListener('click', () => pushSampleNotif(2, 'critical'));
elBtnNotifClear.addEventListener('click', () => {
  if (!engine || !engine.debug_notif_clear) return;
  engine.debug_notif_clear();
  request();
});

elViewportPreset.addEventListener('change', () => {
  const val = elViewportPreset.value;
  elCustomSizeWrap.hidden = val !== 'custom';
  if (val !== 'custom') {
    const [w, h] = val.split('x').map(Number);
    updateViewport(w, h);
    if (isRecording) recordedSteps.push({ type: 'viewport', w, h });
  }
});

for (const inp of [elCustomW, elCustomH]) {
  inp.addEventListener('change', () => {
    const w = Number(elCustomW.value);
    const h = Number(elCustomH.value);
    updateViewport(w, h);
    if (isRecording) recordedSteps.push({ type: 'viewport', w, h });
  });
}

elBtnRotate.addEventListener('click', () => {
  updateViewport(logicalH, logicalW);
  if (isRecording) recordedSteps.push({ type: 'viewport', w: logicalW, h: logicalH });
});

elBtnBack.addEventListener('click', () => {
  if (!engine) return;
  const handled = engine.debug_back();
  if (isRecording) recordedSteps.push({ type: 'back' });
  appendLog(handled ? 'Back handled by app' : 'Back reached root');
  request();
});

elBtnReset.addEventListener('click', () => {
  if (frameTrapped) {
    window.location.reload();
    return;
  }
  if (!engine) return;
  closeEditor();
  engine.debug_reset();
  if (apps[activeAppIdx] === 'files') elFilesMode.value = '0';
  appendLog('Reset app state');
  request();
});

elChkDark.addEventListener('change', () => {
  syncAppThemeUI();
  if (isRecording) recordedSteps.push({ type: 'theme', dark: elChkDark.checked });
  request();
});
elChkReduced.addEventListener('change', request);
elChkBounds.addEventListener('change', () => {
  elChkModalBounds.checked = elChkBounds.checked;
  if (!elChkBounds.checked) highlightNodeId = null;
  drawOverlay(Math.min(devicePixelRatio || 1, 2));
});
elChkModalBounds.addEventListener('change', () => {
  elChkBounds.checked = elChkModalBounds.checked;
  elChkBounds.dispatchEvent(new Event('change'));
});
elChkModalTestBar.addEventListener('change', () => {
  elQuickTestBar.hidden = !elChkModalTestBar.checked;
});
elChkModalInspector.addEventListener('change', () => {
  const hidden = !elChkModalInspector.checked;
  elInspectorPane.hidden = hidden;
  elInspectorSplitter.hidden = hidden;
});

// =============================================================================
// Notion-style Settings Modal & Appearance (Dark / Light / System Theme)
// =============================================================================

const UI_THEME_KEY = 'bhooDebug.uiTheme';
const APP_THEME_KEY = 'bhooDebug.appTheme';
const DOT_GRID_KEY = 'bhooDebug.dotGrid';
const prefersLightMedia = window.matchMedia ? window.matchMedia('(prefers-color-scheme: light)') : null;
let uiThemePref = 'dark';

function applyUiTheme(pref, persist = true) {
  uiThemePref = pref === 'light' || pref === 'system' ? pref : 'dark';
  const resolved = uiThemePref === 'system'
    ? (prefersLightMedia && prefersLightMedia.matches ? 'light' : 'dark')
    : uiThemePref;
  document.documentElement.setAttribute('data-theme', resolved);
  for (const btn of document.querySelectorAll('[data-ui-theme]')) {
    btn.classList.toggle('active', btn.dataset.uiTheme === uiThemePref);
  }
  if (persist) {
    try { localStorage.setItem(UI_THEME_KEY, uiThemePref); } catch {}
  }
}

function syncAppThemeUI() {
  const cur = elChkDark.checked ? 'dark' : 'light';
  for (const btn of document.querySelectorAll('[data-app-theme]')) {
    btn.classList.toggle('active', btn.dataset.appTheme === cur);
  }
}

function applyAppTheme(mode, persist = true) {
  elChkDark.checked = mode !== 'light';
  syncAppThemeUI();
  if (persist) {
    try { localStorage.setItem(APP_THEME_KEY, elChkDark.checked ? 'dark' : 'light'); } catch {}
  }
  if (isRecording) recordedSteps.push({ type: 'theme', dark: elChkDark.checked });
  request();
}

function applyDotGrid(enabled, persist = true) {
  elChkDotGrid.checked = enabled;
  elStageWrap.classList.toggle('no-dots', !enabled);
  if (persist) {
    try { localStorage.setItem(DOT_GRID_KEY, enabled ? '1' : '0'); } catch {}
  }
}

for (const btn of document.querySelectorAll('[data-ui-theme]')) {
  btn.addEventListener('click', () => applyUiTheme(btn.dataset.uiTheme));
}
for (const btn of document.querySelectorAll('[data-app-theme]')) {
  btn.addEventListener('click', () => applyAppTheme(btn.dataset.appTheme));
}
elChkDotGrid.addEventListener('change', () => applyDotGrid(elChkDotGrid.checked));
if (prefersLightMedia && prefersLightMedia.addEventListener) {
  prefersLightMedia.addEventListener('change', () => {
    if (uiThemePref === 'system') applyUiTheme('system', false);
  });
}

const SETTINGS_PAGE_TITLES = {
  appearance: 'Appearance',
  interaction: 'Interaction & Motion',
  fonts: 'Fonts & Persian',
  storage: 'Storage & Reset',
};

function selectSettingsPage(page) {
  elSettingsModalHeading.textContent = SETTINGS_PAGE_TITLES[page] || 'Settings';
  for (const btn of document.querySelectorAll('[data-settings-page]')) {
    btn.classList.toggle('active', btn.dataset.settingsPage === page);
  }
  for (const sec of document.querySelectorAll('[data-settings-view]')) {
    sec.hidden = sec.dataset.settingsView !== page;
  }
}

async function refreshSettingsFontInfo() {
  try {
    const meta = await fetch('/__debug/fonts', { cache: 'no-store' }).then(r => r.json());
    const count = Array.isArray(meta.webPersianFonts) ? meta.webPersianFonts.length : 0;
    elSettingsFontMeta.textContent = `Latin: ${meta.latinFamily || 'Roboto'} · Persian (fa): ${meta.persianFamily || 'Shabnam'} (${count} weights loaded)`;
  } catch {}
}

function openSettingsModal(page = 'appearance') {
  selectSettingsPage(page);
  syncAppThemeUI();
  elChkModalBounds.checked = elChkBounds.checked;
  elSettingsModal.hidden = false;
  refreshSettingsFontInfo();
}

function closeSettingsModal() {
  elSettingsModal.hidden = true;
}

elBtnSettings.addEventListener('click', () => openSettingsModal('appearance'));
elBtnCloseSettings.addEventListener('click', closeSettingsModal);
elSettingsModal.addEventListener('click', e => {
  if (e.target === elSettingsModal) closeSettingsModal();
});
for (const btn of document.querySelectorAll('[data-settings-page]')) {
  btn.addEventListener('click', () => selectSettingsPage(btn.dataset.settingsPage));
}
elBtnResetLayout.addEventListener('click', () => {
  setInspectorWidth(DEFAULT_INSPECTOR_WIDTH, false);
  setStateBoxHeight(DEFAULT_STATE_HEIGHT, false);
  try {
    localStorage.removeItem(INSPECTOR_WIDTH_KEY);
    localStorage.removeItem(STATE_HEIGHT_KEY);
  } catch {}
  appendLog('Reset Inspector panel layout');
});
elBtnResetTests.addEventListener('click', () => syncRepoTests(true));
elBtnClearAllLogs.addEventListener('click', () => {
  elActionLog.innerHTML = '';
  busEvents.length = 0;
  unreadBusCount = 0;
  elBusBadge.hidden = true;
  renderBusList();
});

try {
  const savedUi = localStorage.getItem(UI_THEME_KEY);
  applyUiTheme(savedUi || 'dark', false);
  const savedApp = localStorage.getItem(APP_THEME_KEY);
  if (savedApp === 'light' || savedApp === 'dark') {
    elChkDark.checked = savedApp === 'dark';
    syncAppThemeUI();
  }
  const savedDots = localStorage.getItem(DOT_GRID_KEY);
  applyDotGrid(savedDots !== '0', false);
} catch {
  applyUiTheme('dark', false);
}

// Resizable vertical & horizontal splitters
const INSPECTOR_WIDTH_KEY = 'bhooDebug.inspectorWidth';
const STATE_HEIGHT_KEY = 'bhooDebug.stateHeight';
const DEFAULT_INSPECTOR_WIDTH = 390;
const DEFAULT_STATE_HEIGHT = 160;

function setInspectorWidth(px, persist = true) {
  const wbWidth = elWorkbench ? elWorkbench.getBoundingClientRect().width : window.innerWidth;
  const maxW = Math.max(280, wbWidth - 260);
  const clamped = Math.round(Math.max(240, Math.min(maxW, px)));
  elInspectorPane.style.width = `${clamped}px`;
  if (persist) {
    try {
      localStorage.setItem(INSPECTOR_WIDTH_KEY, String(clamped));
    } catch {}
  }
}

function setStateBoxHeight(px, persist = true) {
  const tabBody = document.getElementById('tabState');
  const avail = tabBody ? tabBody.getBoundingClientRect().height : 420;
  const maxH = Math.max(120, avail - 110);
  const clamped = Math.round(Math.max(64, Math.min(maxH, px)));
  elAppStateJson.style.height = `${clamped}px`;
  if (persist) {
    try {
      localStorage.setItem(STATE_HEIGHT_KEY, String(clamped));
    } catch {}
  }
}

try {
  const savedW = Number(localStorage.getItem(INSPECTOR_WIDTH_KEY));
  if (Number.isFinite(savedW) && savedW >= 240) setInspectorWidth(savedW, false);
  const savedH = Number(localStorage.getItem(STATE_HEIGHT_KEY));
  if (Number.isFinite(savedH) && savedH >= 64) setStateBoxHeight(savedH, false);
} catch {}

let resizingCol = false;
elInspectorSplitter.addEventListener('pointerdown', e => {
  if (e.pointerType === 'mouse' && e.button !== 0) return;
  e.preventDefault();
  resizingCol = true;
  elInspectorSplitter.classList.add('dragging');
  document.body.classList.add('resizing-col');
  try { elInspectorSplitter.setPointerCapture(e.pointerId); } catch {}
});
elInspectorSplitter.addEventListener('pointermove', e => {
  if (!resizingCol) return;
  const rect = elWorkbench.getBoundingClientRect();
  setInspectorWidth(rect.right - e.clientX);
});
for (const evName of ['pointerup', 'pointercancel']) {
  elInspectorSplitter.addEventListener(evName, () => {
    if (!resizingCol) return;
    resizingCol = false;
    elInspectorSplitter.classList.remove('dragging');
    document.body.classList.remove('resizing-col');
  });
}
elInspectorSplitter.addEventListener('dblclick', () => {
  setInspectorWidth(DEFAULT_INSPECTOR_WIDTH, false);
  try { localStorage.removeItem(INSPECTOR_WIDTH_KEY); } catch {}
});
elInspectorSplitter.addEventListener('keydown', e => {
  if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
    e.preventDefault();
    const cur = elInspectorPane.getBoundingClientRect().width;
    const step = e.shiftKey ? 40 : 16;
    setInspectorWidth(cur + (e.key === 'ArrowLeft' ? step : -step));
  }
});

let resizingRow = false;
let rowStartY = 0;
let rowStartH = DEFAULT_STATE_HEIGHT;
elStateLogSplitter.addEventListener('pointerdown', e => {
  if (e.pointerType === 'mouse' && e.button !== 0) return;
  e.preventDefault();
  resizingRow = true;
  rowStartY = e.clientY;
  rowStartH = elAppStateJson.getBoundingClientRect().height;
  elStateLogSplitter.classList.add('dragging');
  document.body.classList.add('resizing-row');
  try { elStateLogSplitter.setPointerCapture(e.pointerId); } catch {}
});
elStateLogSplitter.addEventListener('pointermove', e => {
  if (!resizingRow) return;
  setStateBoxHeight(rowStartH + (e.clientY - rowStartY));
});
for (const evName of ['pointerup', 'pointercancel']) {
  elStateLogSplitter.addEventListener(evName, () => {
    if (!resizingRow) return;
    resizingRow = false;
    elStateLogSplitter.classList.remove('dragging');
    document.body.classList.remove('resizing-row');
  });
}
elStateLogSplitter.addEventListener('dblclick', () => {
  setStateBoxHeight(DEFAULT_STATE_HEIGHT, false);
  try { localStorage.removeItem(STATE_HEIGHT_KEY); } catch {}
});
elStateLogSplitter.addEventListener('keydown', e => {
  if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
    e.preventDefault();
    const cur = elAppStateJson.getBoundingClientRect().height;
    const step = e.shiftKey ? 40 : 16;
    setStateBoxHeight(cur + (e.key === 'ArrowDown' ? step : -step));
  }
});

window.addEventListener('resize', () => {
  if (!elInspectorPane.hidden) {
    setInspectorWidth(elInspectorPane.getBoundingClientRect().width, false);
  }
});

elBtnClearLog.addEventListener('click', () => {
  elActionLog.innerHTML = '';
});
elSemFilter.addEventListener('input', refreshSemanticsTree);

const inspectorTabs = [...document.querySelectorAll('.tabs > .tab')];
for (const [index, tab] of inspectorTabs.entries()) {
  tab.addEventListener('click', () => switchTab(tab.dataset.tab));
  tab.addEventListener('keydown', event => {
    const last = inspectorTabs.length - 1;
    const nextIndex = {
      ArrowLeft: index === 0 ? last : index - 1,
      ArrowRight: index === last ? 0 : index + 1,
      Home: 0,
      End: last,
    }[event.key];
    if (nextIndex === undefined) return;
    event.preventDefault();
    const next = inspectorTabs[nextIndex];
    switchTab(next.dataset.tab);
    next.focus();
  });
}

// Canvas input forwarding + optional test step recording
function localPointer(e) {
  const r = appCanvas.getBoundingClientRect();
  return [e.clientX - r.left, e.clientY - r.top];
}

appCanvas.addEventListener('contextmenu', e => e.preventDefault());
for (const type of ['pointerdown', 'pointermove', 'pointerup', 'pointercancel']) {
  appCanvas.addEventListener(type, e => {
    if (type === 'pointerdown' && e.pointerType === 'mouse' && e.button !== 0) return;
    pointer = localPointer(e);
    elMetaPointer.textContent = `pointer: ${Math.round(pointer[0])}, ${Math.round(pointer[1])}`;
    updateHoverInspector();

    if (type === 'pointerdown') {
      pointerDownPos = [...pointer];
      dragMoved = false;
      closeEditor();
      appCanvas.focus();
      try { appCanvas.setPointerCapture(e.pointerId); } catch {}
    } else if (type === 'pointermove' && pointerDownPos) {
      if (Math.hypot(pointer[0] - pointerDownPos[0], pointer[1] - pointerDownPos[1]) >= 8) {
        dragMoved = true;
      }
    }

    const pointerBit = elChkTouch.checked || e.pointerType === 'touch' ? 128 : e.pointerType === 'pen' ? 256 : 0;
    const edgeBit = { pointerdown: 1, pointerup: 2, pointercancel: 4 }[type] || 0;
    push(pointerBit | edgeBit);

    if (type === 'pointerup') {
      const wasDrag = dragMoved;
      pointerDownPos = null;
      dragMoved = false;
      if (!wasDrag) {
        const [px, py] = pointer;
        if (isRecording) {
          let hit = null;
          for (let i = nodes.length - 1; i >= 0; i--) {
            const n = nodes[i];
            if (px >= n.bounds[0] && px < n.bounds[0] + n.bounds[2] && py >= n.bounds[1] && py < n.bounds[1] + n.bounds[3]) {
              hit = n;
              break;
            }
          }
          if (hit && hit.label) {
            recordedSteps.push({ type: 'tapNode', label: hit.label });
          } else {
            recordedSteps.push({ type: 'tap', x: Math.round(px), y: Math.round(py) });
          }
        }
        const field = nodes.find(
          n => n.role === 'text field' && px >= n.bounds[0] && px < n.bounds[0] + n.bounds[2] && py >= n.bounds[1] && py < n.bounds[1] + n.bounds[3]
        );
        if (field) openEditor(field.id, field.label);
      }
    } else if (type === 'pointercancel') {
      pointerDownPos = null;
      dragMoved = false;
    }
  });
}

appCanvas.addEventListener(
  'wheel',
  e => {
    e.preventDefault();
    pointer = localPointer(e);
    const scale = e.deltaMode === 1 ? 40 : e.deltaMode === 2 ? logicalH : 1;
    const horiz = e.shiftKey || Math.abs(e.deltaX) > Math.abs(e.deltaY);
    push(horiz ? 512 : 0, 0, (horiz ? e.deltaX || e.deltaY : e.deltaY) * scale);
  },
  { passive: false }
);

// =============================================================================
// Visual Feedback Draw Mode (Alt+F) + Compact Bottom Dock + AI Copy / Image
// =============================================================================

let drawMode = false;
let drawTool = 'box'; // 'box' | 'freehand' | 'arrow'
let drawColor = '#ff2e63';
let isDrawingAnnotation = false;
let activeDraft = null; // { tool, color, x1, y1, x2, y2, points: [[x,y], ...] }
let feedbackSeq = 1;
const feedbackItems = []; // { id, num, app, appName, desktopId, viewport, theme, text, annotation, bounds, targets, stateSnapshot, imageDataUrl, imageBlob, createdAt }
let previewFeedbackId = null;
let toastTimer = 0;

function showFeedbackToast(msg) {
  clearTimeout(toastTimer);
  elFeedbackToast.textContent = msg;
  elFeedbackToast.hidden = false;
  toastTimer = setTimeout(() => {
    elFeedbackToast.hidden = true;
  }, 2400);
}

function clampToCanvas(e) {
  const r = appCanvas.getBoundingClientRect();
  const x = Math.max(0, Math.min(logicalW, e.clientX - r.left));
  const y = Math.max(0, Math.min(logicalH, e.clientY - r.top));
  return [x, y];
}

function annotationBounds(ann) {
  if (!ann) return [0, 0, 0, 0];
  if (ann.tool === 'freehand' && ann.points && ann.points.length) {
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const [px, py] of ann.points) {
      if (px < minX) minX = px;
      if (py < minY) minY = py;
      if (px > maxX) maxX = px;
      if (py > maxY) maxY = py;
    }
    return [minX, minY, Math.max(4, maxX - minX), Math.max(4, maxY - minY)];
  }
  const x = Math.min(ann.x1, ann.x2);
  const y = Math.min(ann.y1, ann.y2);
  const w = Math.abs(ann.x2 - ann.x1);
  const h = Math.abs(ann.y2 - ann.y1);
  if (w < 6 && h < 6) {
    return [Math.max(0, x - 16), Math.max(0, y - 16), 32, 32];
  }
  return [x, y, w, h];
}

function findOverlappingNodes(bounds) {
  const [bx, by, bw, bh] = bounds;
  const hits = [];
  for (const n of nodes) {
    const [nx, ny, nw, nh] = n.bounds;
    if (nw <= 0 || nh <= 0) continue;
    const interX = Math.max(0, Math.min(bx + bw, nx + nw) - Math.max(bx, nx));
    const interY = Math.max(0, Math.min(by + bh, ny + nh) - Math.max(by, ny));
    if (interX > 0 && interY > 0) {
      hits.push({
        id: n.id,
        role: n.role,
        label: n.label || '',
        bounds: n.bounds.map(v => Math.round(v)),
      });
    }
  }
  return hits;
}

function drawAnnotationShape(targetCtx, ann, numLabel = null, scale = 1) {
  if (!ann) return;
  const color = ann.color || '#ff2e63';
  const [bx, by, bw, bh] = annotationBounds(ann);
  const isPoint = ann.tool === 'box' && Math.abs(ann.x2 - ann.x1) < 6 && Math.abs(ann.y2 - ann.y1) < 6;

  targetCtx.save();
  targetCtx.lineCap = 'round';
  targetCtx.lineJoin = 'round';

  const strokeWithHalo = drawPath => {
    targetCtx.beginPath();
    drawPath();
    targetCtx.strokeStyle = 'rgba(0, 0, 0, 0.55)';
    targetCtx.lineWidth = 5 * scale;
    targetCtx.stroke();

    targetCtx.beginPath();
    drawPath();
    targetCtx.strokeStyle = color;
    targetCtx.lineWidth = 2.6 * scale;
    targetCtx.stroke();
  };

  if (isPoint) {
    const cx = ann.x1 * scale;
    const cy = ann.y1 * scale;
    const rad = 16 * scale;
    targetCtx.fillStyle = `${color}28`;
    targetCtx.beginPath();
    targetCtx.arc(cx, cy, rad, 0, Math.PI * 2);
    targetCtx.fill();
    strokeWithHalo(() => targetCtx.arc(cx, cy, rad, 0, Math.PI * 2));
  } else if (ann.tool === 'box') {
    const x = bx * scale, y = by * scale, w = bw * scale, h = bh * scale;
    const r = Math.min(8 * scale, w / 4, h / 4);
    targetCtx.fillStyle = `${color}20`;
    targetCtx.beginPath();
    targetCtx.roundRect(x, y, w, h, r);
    targetCtx.fill();
    strokeWithHalo(() => targetCtx.roundRect(x, y, w, h, r));
  } else if (ann.tool === 'freehand' && ann.points && ann.points.length > 1) {
    strokeWithHalo(() => {
      targetCtx.moveTo(ann.points[0][0] * scale, ann.points[0][1] * scale);
      for (let i = 1; i < ann.points.length; i++) {
        targetCtx.lineTo(ann.points[i][0] * scale, ann.points[i][1] * scale);
      }
    });
  } else if (ann.tool === 'arrow') {
    const x1 = ann.x1 * scale, y1 = ann.y1 * scale, x2 = ann.x2 * scale, y2 = ann.y2 * scale;
    const angle = Math.atan2(y2 - y1, x2 - x1);
    const head = 14 * scale;
    strokeWithHalo(() => {
      targetCtx.moveTo(x1, y1);
      targetCtx.lineTo(x2, y2);
      targetCtx.moveTo(x2, y2);
      targetCtx.lineTo(x2 - head * Math.cos(angle - Math.PI / 6), y2 - head * Math.sin(angle - Math.PI / 6));
      targetCtx.moveTo(x2, y2);
      targetCtx.lineTo(x2 - head * Math.cos(angle + Math.PI / 6), y2 - head * Math.sin(angle + Math.PI / 6));
    });
  }

  if (numLabel !== null) {
    const badgeText = `#${numLabel}`;
    targetCtx.font = `700 ${11 * scale}px ui-monospace, SFMono-Regular, Menlo, monospace`;
    const tw = targetCtx.measureText(badgeText).width;
    const pw = Math.max(22 * scale, tw + 10 * scale);
    const ph = 18 * scale;
    const px = Math.max(4 * scale, Math.min(logicalW * scale - pw - 4 * scale, bx * scale));
    const py = by * scale > ph + 6 * scale ? by * scale - ph - 4 * scale : by * scale + 4 * scale;
    targetCtx.fillStyle = color;
    targetCtx.beginPath();
    targetCtx.roundRect(px, py, pw, ph, 9 * scale);
    targetCtx.fill();
    targetCtx.strokeStyle = '#ffffff';
    targetCtx.lineWidth = 1.5 * scale;
    targetCtx.stroke();
    targetCtx.fillStyle = '#ffffff';
    targetCtx.textAlign = 'center';
    targetCtx.textBaseline = 'middle';
    targetCtx.fillText(badgeText, px + pw / 2, py + ph / 2 + 0.5 * scale);
  }
  targetCtx.restore();
}

function renderFeedbackCanvas() {
  const dpr = Math.min(devicePixelRatio || 1, 2);
  const w = Math.round(logicalW * dpr);
  const h = Math.round(logicalH * dpr);
  if (feedbackCanvas.width !== w || feedbackCanvas.height !== h) {
    feedbackCanvas.width = w;
    feedbackCanvas.height = h;
  }
  fctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  fctx.clearRect(0, 0, logicalW, logicalH);

  // Subtle translucent dim while actively in Draw Mode so markings stand out.
  if (drawMode) {
    fctx.fillStyle = 'rgba(0, 0, 0, 0.08)';
    fctx.fillRect(0, 0, logicalW, logicalH);
  }

  if (activeDraft) {
    drawAnnotationShape(fctx, activeDraft, feedbackSeq, 1);
  }
}

function setDrawMode(enabled) {
  drawMode = Boolean(enabled);
  elStageWrap.classList.toggle('draw-mode', drawMode);
  elDrawModeHud.hidden = !drawMode;
  if (!drawMode) {
    isDrawingAnnotation = false;
    activeDraft = null;
    elFeedbackPopover.hidden = true;
  } else {
    if (!elAppPickerModal.hidden) closeAppPickerModal();
    if (!elSettingsModal.hidden) closeSettingsModal();
    closeEditor();
  }
  renderFeedbackCanvas();
}

function positionFeedbackPopover(bounds) {
  const stageRect = elStageWrap.getBoundingClientRect();
  const canvasRect = appCanvas.getBoundingClientRect();
  const popW = Math.min(310, stageRect.width - 24);
  const popH = 148;
  const targetRight = canvasRect.left - stageRect.left + bounds[0] + bounds[2];
  const targetLeft = canvasRect.left - stageRect.left + bounds[0];
  const targetBottom = canvasRect.top - stageRect.top + bounds[1] + bounds[3];
  const targetTop = canvasRect.top - stageRect.top + bounds[1];

  let left = targetRight + 12;
  if (left + popW > stageRect.width - 12) {
    left = targetLeft - popW - 12;
  }
  if (left < 12) {
    left = Math.max(12, Math.min(stageRect.width - popW - 12, targetLeft));
  }

  const dockH = elFeedbackDock.hidden ? 0 : elFeedbackDock.getBoundingClientRect().height;
  let top = targetTop;
  if (top + popH > stageRect.height - dockH - 12) {
    top = Math.max(58, Math.min(stageRect.height - dockH - popH - 12, targetBottom - popH));
  }
  top = Math.max(58, top);

  elFeedbackPopover.style.left = `${Math.round(left)}px`;
  elFeedbackPopover.style.top = `${Math.round(top)}px`;
}

function openFeedbackDraftPopover(draft) {
  const bounds = annotationBounds(draft);
  const targets = findOverlappingNodes(bounds);
  elFbDraftNum.textContent = `#${feedbackSeq}`;
  elFbDraftNum.style.background = draft.color;
  const targetSummary = targets.length
    ? targets.slice(0, 2).map(t => (t.label ? `"${t.label}"` : t.role)).join(', ')
    : `${Math.round(bounds[2])}×${Math.round(bounds[3])} lp @ (${Math.round(bounds[0])}, ${Math.round(bounds[1])})`;
  elFbDraftTarget.textContent = targetSummary;
  elFeedbackInput.value = '';
  elFeedbackPopover.hidden = false;
  positionFeedbackPopover(bounds);
  requestAnimationFrame(() => elFeedbackInput.focus());
}

function cancelFeedbackDraft() {
  activeDraft = null;
  elFeedbackPopover.hidden = true;
  renderFeedbackCanvas();
}

function canvasToBlob(canvas) {
  return new Promise(resolve => canvas.toBlob(blob => resolve(blob), 'image/png'));
}

function wrapCanvasText(c, text, maxWidth) {
  const words = String(text || '').split(/\s+/);
  const lines = [];
  let line = '';
  for (const w of words) {
    const test = line ? `${line} ${w}` : w;
    if (c.measureText(test).width > maxWidth && line) {
      lines.push(line);
      line = w;
    } else {
      line = test;
    }
  }
  if (line) lines.push(line);
  return lines.slice(0, 3);
}

async function captureAnnotatedSnapshot(ann, num, text, appMeta) {
  const scale = 2;
  const headerH = 34 * scale;
  const footerLines = text ? Math.min(3, Math.max(1, Math.ceil(text.length / 44))) : 0;
  const footerH = footerLines ? (16 + footerLines * 16) * scale : 0;
  const out = document.createElement('canvas');
  out.width = logicalW * scale;
  out.height = headerH + logicalH * scale + footerH;
  const oc = out.getContext('2d');

  // Crisp header strip so AI immediately knows app, viewport & annotation number
  oc.fillStyle = '#11141c';
  oc.fillRect(0, 0, out.width, out.height);
  oc.fillStyle = ann.color || '#ff2e63';
  oc.beginPath();
  oc.roundRect(10 * scale, 7 * scale, 26 * scale, 20 * scale, 10 * scale);
  oc.fill();
  oc.fillStyle = '#ffffff';
  oc.font = `700 ${11 * scale}px ui-monospace, SFMono-Regular, Menlo, monospace`;
  oc.textAlign = 'center';
  oc.textBaseline = 'middle';
  oc.fillText(`#${num}`, 23 * scale, 17 * scale);

  oc.textAlign = 'left';
  oc.fillStyle = '#e6e8ef';
  oc.font = `600 ${12 * scale}px system-ui, -apple-system, "Shabnam", sans-serif`;
  oc.fillText(`${appMeta.name} (${appMeta.desktopId})`, 44 * scale, 17 * scale);

  oc.textAlign = 'right';
  oc.fillStyle = '#939bb0';
  oc.font = `500 ${11 * scale}px ui-monospace, SFMono-Regular, Menlo, monospace`;
  oc.fillText(`${logicalW}×${logicalH} lp`, out.width - 10 * scale, 17 * scale);

  // Draw live Bhoo app canvas + user annotation
  oc.save();
  oc.translate(0, headerH);
  oc.drawImage(appCanvas, 0, 0, logicalW * scale, logicalH * scale);
  drawAnnotationShape(oc, ann, num, scale);
  oc.restore();

  // Draw feedback caption strip at the bottom of the PNG
  if (footerH > 0) {
    const fy = headerH + logicalH * scale;
    oc.fillStyle = '#161a24';
    oc.fillRect(0, fy, out.width, footerH);
    oc.fillStyle = ann.color || '#ff2e63';
    oc.fillRect(0, fy, 4 * scale, footerH);
    oc.fillStyle = '#eef1f8';
    oc.font = `500 ${12 * scale}px system-ui, -apple-system, "Shabnam", sans-serif`;
    oc.textAlign = 'left';
    oc.textBaseline = 'top';
    const wrapped = wrapCanvasText(oc, text, out.width - 24 * scale);
    wrapped.forEach((ln, idx) => {
      oc.fillText(ln, 12 * scale, fy + (8 + idx * 16) * scale);
    });
  }

  const blob = await canvasToBlob(out);
  const dataUrl = out.toDataURL('image/png');
  return { blob, dataUrl, width: out.width, height: out.height };
}

function loadImgElement(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}

async function buildCombinedFeedbackImage(items) {
  if (!items.length) return null;
  if (items.length === 1) {
    return { blob: items[0].imageBlob, dataUrl: items[0].imageDataUrl };
  }
  const imgs = await Promise.all(items.map(it => loadImgElement(it.imageDataUrl)));
  const cols = Math.min(3, imgs.length);
  const rows = Math.ceil(imgs.length / cols);
  const pad = 24;
  const maxTileW = Math.min(640, Math.max(...imgs.map(i => i.width)));
  const tileDims = imgs.map(i => {
    const ratio = Math.min(1, maxTileW / i.width);
    return { img: i, w: Math.round(i.width * ratio), h: Math.round(i.height * ratio) };
  });
  const colW = Math.max(...tileDims.map(d => d.w));
  const rowH = Math.max(...tileDims.map(d => d.h));
  const out = document.createElement('canvas');
  out.width = cols * colW + (cols + 1) * pad;
  out.height = rows * rowH + (rows + 1) * pad;
  const oc = out.getContext('2d');
  oc.fillStyle = '#0c0e14';
  oc.fillRect(0, 0, out.width, out.height);

  tileDims.forEach((d, idx) => {
    const col = idx % cols;
    const row = Math.floor(idx / cols);
    const x = pad + col * (colW + pad) + Math.round((colW - d.w) / 2);
    const y = pad + row * (rowH + pad);
    oc.drawImage(d.img, x, y, d.w, d.h);
    oc.strokeStyle = '#2d3548';
    oc.lineWidth = 2;
    oc.strokeRect(x, y, d.w, d.h);
  });

  const blob = await canvasToBlob(out);
  return { blob, dataUrl: out.toDataURL('image/png') };
}

function buildAiFeedbackMarkdown(items) {
  if (!items.length) return '';
  const lines = ['## SpectreOS Bhoo UI Feedback', ''];
  for (const it of items) {
    const [bx, by, bw, bh] = it.bounds.map(v => Math.round(v));
    lines.push(`### #${it.num} — ${it.appName} (\`${it.desktopId}\`) · ${it.viewport} · ${it.theme} theme`);
    lines.push(`- **Feedback:** ${it.text}`);
    lines.push(`- **Marked Region (${it.annotation.tool}):** \`x=${bx}, y=${by}, w=${bw}, h=${bh}\` lp`);
    if (it.targets.length) {
      const targetStr = it.targets
        .slice(0, 6)
        .map(t => `\`${t.role}${t.label ? ` "${t.label}"` : ''} [${t.bounds.join(',')}]\``)
        .join(', ');
      lines.push(`- **UI Targets in Region:** ${targetStr}`);
    }
    if (it.stateSnapshot && Object.keys(it.stateSnapshot).length) {
      lines.push(`- **App State:** \`${JSON.stringify(it.stateSnapshot)}\``);
    }
    lines.push('');
  }
  return lines.join('\n').trim();
}

async function copyTextToClipboard(text, label) {
  try {
    await navigator.clipboard.writeText(text);
    showFeedbackToast(label);
    appendLog(label);
  } catch {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.append(ta);
    ta.select();
    try {
      if (!document.execCommand('copy')) throw new Error('Copy failed');
      showFeedbackToast(label);
      appendLog(label);
    } catch {
      showFeedbackToast('Clipboard blocked — open preview to copy text manually');
    }
    ta.remove();
  }
}

function downloadBlob(blob, filename) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function copyImageBlobToClipboard(blob, fallbackName, label) {
  if (!blob) return;
  try {
    if (navigator.clipboard && window.ClipboardItem) {
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
      showFeedbackToast(label);
      appendLog(label);
      return;
    }
    throw new Error('ClipboardItem unavailable');
  } catch {
    downloadBlob(blob, fallbackName);
    showFeedbackToast(`Clipboard image unavailable — downloaded ${fallbackName}`);
    appendLog(`Downloaded ${fallbackName}`);
  }
}

async function commitFeedbackDraft() {
  if (!activeDraft) return;
  const text = elFeedbackInput.value.trim();
  if (!text) {
    elFeedbackInput.focus();
    return;
  }
  const num = feedbackSeq++;
  const slug = currentAppName();
  const appMeta = getAppMeta(slug);
  const bounds = annotationBounds(activeDraft);
  const targets = findOverlappingNodes(bounds);
  const annCopy = structuredClone(activeDraft);
  const snap = await captureAnnotatedSnapshot(annCopy, num, text, appMeta);

  const item = {
    id: `fb-${Date.now()}-${num}`,
    num,
    app: slug,
    appName: appMeta.name,
    desktopId: appMeta.desktopId,
    viewport: `${logicalW}×${logicalH} lp`,
    theme: elChkDark.checked ? 'dark' : 'light',
    text,
    annotation: annCopy,
    bounds,
    targets,
    stateSnapshot: structuredClone(lastParsedState || {}),
    imageDataUrl: snap.dataUrl,
    imageBlob: snap.blob,
    createdAt: new Date().toLocaleTimeString('en-GB', { hour12: false }),
  };

  feedbackItems.push(item);
  setDrawMode(false);
  renderFeedbackDock();
  showFeedbackToast(`Saved feedback #${num} — use Copy for AI or Copy Image below`);
  appendLog(`Feedback #${num} (${appMeta.name}): ${text}`);
}

function removeFeedbackItem(id) {
  const idx = feedbackItems.findIndex(f => f.id === id);
  if (idx >= 0) {
    feedbackItems.splice(idx, 1);
    if (previewFeedbackId === id) closeFeedbackPreview();
    renderFeedbackDock();
  }
}

function openFeedbackPreview(id) {
  const item = feedbackItems.find(f => f.id === id);
  if (!item) return;
  previewFeedbackId = id;
  elFbPreviewNum.textContent = `#${item.num}`;
  elFbPreviewNum.style.background = item.annotation.color || '#ff2e63';
  elFbPreviewHeading.textContent = `${item.appName} — Feedback #${item.num}`;
  elFbPreviewMeta.textContent = `${item.desktopId} · ${item.viewport} · ${item.theme} theme · ${item.createdAt}`;
  elFbPreviewImg.src = item.imageDataUrl;
  elFbPreviewPrompt.textContent = buildAiFeedbackMarkdown([item]);
  elFeedbackPreviewModal.hidden = false;
}

function closeFeedbackPreview() {
  previewFeedbackId = null;
  elFeedbackPreviewModal.hidden = true;
}

function renderFeedbackDock() {
  elFeedbackCountBadge.textContent = String(feedbackItems.length);
  elFeedbackDock.hidden = feedbackItems.length === 0;
  elFeedbackChipList.innerHTML = '';

  for (const item of feedbackItems) {
    const card = document.createElement('div');
    card.className = 'feedback-mini-card';
    card.innerHTML = `
      <button type="button" class="feedback-thumb-btn" title="Open full annotated preview">
        <img src="${item.imageDataUrl}" alt="Feedback #${item.num}">
        <span class="feedback-thumb-badge" style="background:${item.annotation.color || '#ff2e63'}">#${item.num}</span>
      </button>
      <div class="feedback-mini-body" title="${escapeHtml(item.text)}">
        <div class="feedback-mini-meta">${escapeHtml(item.appName)} · ${escapeHtml(item.viewport)}</div>
        <div class="feedback-mini-text">${escapeHtml(item.text)}</div>
      </div>
      <div class="feedback-mini-btns">
        <button type="button" class="feedback-mini-icon" data-act="copy-text" title="Copy feedback text for AI">${heroicon('clipboard-document')}</button>
        <button type="button" class="feedback-mini-icon" data-act="copy-img" title="Copy annotated image">${heroicon('photo')}</button>
      </div>
      <button type="button" class="feedback-mini-icon" data-act="delete" title="Remove feedback">${heroicon('x-mark')}</button>
    `;
    card.querySelector('.feedback-thumb-btn').addEventListener('click', () => openFeedbackPreview(item.id));
    card.querySelector('.feedback-mini-body').addEventListener('click', () => openFeedbackPreview(item.id));
    card.querySelector('[data-act="copy-text"]').addEventListener('click', () => {
      copyTextToClipboard(buildAiFeedbackMarkdown([item]), `Copied feedback #${item.num} prompt for AI`);
    });
    card.querySelector('[data-act="copy-img"]').addEventListener('click', () => {
      copyImageBlobToClipboard(item.imageBlob, `bhoo-feedback-${item.app}-${item.num}.png`, `Copied feedback #${item.num} image — paste into AI!`);
    });
    card.querySelector('[data-act="delete"]').addEventListener('click', () => removeFeedbackItem(item.id));
    elFeedbackChipList.append(card);
  }
  elFeedbackChipList.scrollLeft = elFeedbackChipList.scrollWidth;
}

// Focus stage cleanly when clicked so Alt+F feels immediate
elStageWrap.addEventListener('pointerdown', e => {
  if (e.target.closest('#feedbackPopover') || e.target.closest('#feedbackDock') || e.target.closest('#drawModeHud')) return;
  if (document.activeElement !== elStageWrap && document.activeElement !== appCanvas) {
    elStageWrap.focus({ preventScroll: true });
  }
});

// Drawing interaction across stage-scroll & feedbackCanvas while Draw Mode is active
function onDrawPointerDown(e) {
  if (!drawMode) return;
  if (e.pointerType === 'mouse' && e.button !== 0) return;
  if (e.target.closest('#feedbackPopover') || e.target.closest('#drawModeHud') || e.target.closest('#feedbackDock')) return;
  e.preventDefault();
  e.stopPropagation();
  elFeedbackPopover.hidden = true;
  const [x, y] = clampToCanvas(e);
  isDrawingAnnotation = true;
  activeDraft = {
    tool: drawTool,
    color: drawColor,
    x1: x,
    y1: y,
    x2: x,
    y2: y,
    points: [[x, y]],
  };
  try { e.currentTarget.setPointerCapture(e.pointerId); } catch {}
  renderFeedbackCanvas();
}

function onDrawPointerMove(e) {
  if (!drawMode || !isDrawingAnnotation || !activeDraft) return;
  e.preventDefault();
  const [x, y] = clampToCanvas(e);
  activeDraft.x2 = x;
  activeDraft.y2 = y;
  if (activeDraft.tool === 'freehand') {
    const last = activeDraft.points[activeDraft.points.length - 1];
    if (!last || Math.hypot(x - last[0], y - last[1]) >= 2) {
      activeDraft.points.push([x, y]);
    }
  }
  renderFeedbackCanvas();
}

function onDrawPointerUp(e) {
  if (!drawMode || !isDrawingAnnotation || !activeDraft) return;
  e.preventDefault();
  isDrawingAnnotation = false;
  const [x, y] = clampToCanvas(e);
  activeDraft.x2 = x;
  activeDraft.y2 = y;
  renderFeedbackCanvas();
  openFeedbackDraftPopover(activeDraft);
}

elStageScroll.addEventListener('pointerdown', onDrawPointerDown);
elStageScroll.addEventListener('pointermove', onDrawPointerMove);
elStageScroll.addEventListener('pointerup', onDrawPointerUp);
elStageScroll.addEventListener('pointercancel', () => {
  if (isDrawingAnnotation) {
    isDrawingAnnotation = false;
    activeDraft = null;
    renderFeedbackCanvas();
  }
});

elBtnStageDraw.addEventListener('click', () => setDrawMode(true));
elBtnDockDraw.addEventListener('click', () => setDrawMode(true));
elBtnExitDrawMode.addEventListener('click', () => setDrawMode(false));

const drawToolButtons = [...elDrawModeHud.querySelectorAll('[data-draw-tool]')];
for (const btn of drawToolButtons) {
  btn.addEventListener('click', () => {
    drawTool = btn.dataset.drawTool;
    for (const b of drawToolButtons) {
      b.classList.toggle('active', b === btn);
    }
  });
}
for (const btn of document.querySelectorAll('[data-draw-color]')) {
  btn.addEventListener('click', () => {
    drawColor = btn.dataset.drawColor;
    for (const b of document.querySelectorAll('[data-draw-color]')) {
      b.classList.toggle('active', b === btn);
    }
    if (activeDraft) {
      activeDraft.color = drawColor;
      elFbDraftNum.style.background = drawColor;
      renderFeedbackCanvas();
    }
  });
}

elBtnSaveFeedbackDraft.addEventListener('click', commitFeedbackDraft);
elBtnCancelFeedbackDraft.addEventListener('click', cancelFeedbackDraft);
elFeedbackInput.addEventListener('keydown', e => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    commitFeedbackDraft();
  } else if (e.key === 'Escape') {
    e.preventDefault();
    cancelFeedbackDraft();
  }
});

elBtnCopyAllAiText.addEventListener('click', async () => {
  if (!feedbackItems.length) return;
  const items = [...feedbackItems];
  const prompt = buildAiFeedbackMarkdown(items);
  try {
    // Start the clipboard write while the click still grants user activation.
    await navigator.clipboard.write([new ClipboardItem({
      'text/plain': new Blob([prompt], { type: 'text/plain' }),
      'image/png': buildCombinedFeedbackImage(items).then(({ blob }) => blob),
    })]);
    const message = 'Copied AI feedback text and annotated screenshot';
    showFeedbackToast(message);
    appendLog(message);
  } catch {
    await copyTextToClipboard(prompt, 'Copied AI feedback text. Use Copy Image to copy the screenshot.');
  }
});

elBtnCopyAllAiImg.addEventListener('click', async () => {
  if (!feedbackItems.length) return;
  const combined = await buildCombinedFeedbackImage(feedbackItems);
  if (!combined) return;
  await copyImageBlobToClipboard(
    combined.blob,
    `bhoo-feedback-${currentAppName()}.png`,
    `Copied annotated PNG (${feedbackItems.length} ${feedbackItems.length === 1 ? 'item' : 'items'}) — Ctrl+V into AI chat!`,
  );
});

elBtnDownloadAllAiImg.addEventListener('click', async () => {
  if (!feedbackItems.length) return;
  const combined = await buildCombinedFeedbackImage(feedbackItems);
  if (combined?.blob) {
    downloadBlob(combined.blob, `bhoo-feedback-${currentAppName()}.png`);
    showFeedbackToast(`Downloaded bhoo-feedback-${currentAppName()}.png`);
  }
});

elBtnClearFeedbacks.addEventListener('click', () => {
  feedbackItems.length = 0;
  feedbackSeq = 1;
  closeFeedbackPreview();
  renderFeedbackDock();
});

elBtnCloseFbPreview.addEventListener('click', closeFeedbackPreview);
elFeedbackPreviewModal.addEventListener('click', e => {
  if (e.target === elFeedbackPreviewModal) closeFeedbackPreview();
});
elBtnPreviewCopyText.addEventListener('click', () => {
  const item = feedbackItems.find(f => f.id === previewFeedbackId);
  if (item) copyTextToClipboard(buildAiFeedbackMarkdown([item]), `Copied feedback #${item.num} prompt for AI`);
});
elBtnPreviewCopyImg.addEventListener('click', () => {
  const item = feedbackItems.find(f => f.id === previewFeedbackId);
  if (item) copyImageBlobToClipboard(item.imageBlob, `bhoo-feedback-${item.app}-${item.num}.png`, `Copied feedback #${item.num} image — paste into AI!`);
});
elBtnPreviewDownloadImg.addEventListener('click', () => {
  const item = feedbackItems.find(f => f.id === previewFeedbackId);
  if (item) {
    downloadBlob(item.imageBlob, `bhoo-feedback-${item.app}-${item.num}.png`);
    showFeedbackToast(`Downloaded bhoo-feedback-${item.app}-${item.num}.png`);
  }
});

window.addEventListener('keydown', e => {
  if (e.altKey && !e.ctrlKey && !e.metaKey && (e.code === 'KeyF' || e.key.toLowerCase() === 'f')) {
    e.preventDefault();
    setDrawMode(!drawMode);
    return;
  }
  if (!elFeedbackPreviewModal.hidden && e.key === 'Escape') {
    e.preventDefault();
    closeFeedbackPreview();
    return;
  }
  if (drawMode && e.key === 'Escape') {
    e.preventDefault();
    if (!elFeedbackPopover.hidden) cancelFeedbackDraft();
    else setDrawMode(false);
    return;
  }
  if (!elAppPickerModal.hidden && e.key === 'Escape') {
    e.preventDefault();
    closeAppPickerModal();
    return;
  }
  if (!elSettingsModal.hidden && e.key === 'Escape') {
    e.preventDefault();
    closeSettingsModal();
    return;
  }
  if (e.target === editor && e.key !== 'Escape') return;
  if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA')) return;
  if (
    drawMode && !isDrawingAnnotation && elFeedbackPopover.hidden &&
    elFeedbackPreviewModal.hidden && elAppPickerModal.hidden && elSettingsModal.hidden &&
    !e.defaultPrevented && !e.isComposing && !e.altKey && !e.ctrlKey && !e.metaKey &&
    !e.target.isContentEditable
  ) {
    const tool = drawToolButtons.find(btn => btn.getAttribute('aria-keyshortcuts') === e.key.toUpperCase());
    if (tool) {
      e.preventDefault();
      tool.click();
      return;
    }
  }
  if ((e.key === 'k' || e.key === 'K') && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    if (elAppPickerModal.hidden) openAppPickerModal();
    else closeAppPickerModal();
    return;
  }
  if (e.key === ',' && !e.ctrlKey && !e.metaKey && !e.altKey) {
    e.preventDefault();
    if (elSettingsModal.hidden) openSettingsModal('appearance');
    else closeSettingsModal();
    return;
  }
  if (e.key === 'Escape') {
    e.preventDefault();
    if (engine && engine.debug_back()) {
      appendLog('Back (Esc)');
      request();
      return;
    }
  }
  if (e.target instanceof HTMLElement && e.target.closest('.hovered-node-panel > summary, #btnReloadWasm')) return;
  const map = {
    Tab: 1,
    Enter: 2,
    ' ': 3,
    ArrowLeft: 4,
    ArrowRight: 5,
    ArrowUp: 6,
    ArrowDown: 7,
    Home: 8,
    End: 9,
    Escape: 10,
    ContextMenu: 11,
    PageUp: 12,
    PageDown: 13,
  };
  const code = map[e.key];
  if (code) {
    e.preventDefault();
    if (isRecording) recordedSteps.push({ type: 'key', key: e.key });
    push(16 | (e.shiftKey ? 64 : 0), code);
  }
});

loadStoredTests();
renderTestUI();
renderBusList();
updateViewport(400, 832);
await Promise.all([preloadPersianFonts(), loadAppCatalog(), syncRepoTests(false), loadWasm(false)]);
void pollVersion();
setInterval(pollVersion, 700);
setInterval(syncClock, 15000);
