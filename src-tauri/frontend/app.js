/* Pezli Editor - Tauri desktop app (plain JavaScript, no bundler) */

const $ = (selector, root = document) => root.querySelector(selector);
const $$ = (selector, root = document) => Array.from(root.querySelectorAll(selector));
const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
const esc = (value) => String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const baseName = (path) => String(path).split(/[/\\]/).pop() || String(path);

function invoke(command, args = {}) {
  const internals = window.__TAURI_INTERNALS__;
  if (!internals || typeof internals.invoke !== 'function') {
    return Promise.reject(new Error('This action requires the Pezli Editor desktop app.'));
  }
  return internals.invoke(command, args);
}

function iconSvg(name, size = 16) {
  const icon = window.feather && window.feather.icons[name];
  return icon ? icon.toSvg({ width: size, height: size, 'aria-hidden': 'true' }) : '';
}

function hydrateIcons(root = document) {
  $$('[data-icon]', root).forEach((el) => {
    el.innerHTML = iconSvg(el.dataset.icon, Number(el.dataset.size) || 16);
  });
}

const BUILTIN_THEMES = [
  { id: 'light', name: 'Light+', bg: '#ffffff', text: '#1e1e1e', cursor: '#000000', selection: '#add6ff', match: '#ffe792', chrome: '#f3f3f3', chromeText: '#252526', border: '#dcdcdc', borderStrong: '#c6c6c6', hover: '#e8e8e8', accent: '#007acc', input: '#ffffff', tabInactive: '#ececec', lineBg: '#f5f5f5', muted: '#6c6c6c' },
  { id: 'dark+', name: 'Dark+', bg: '#1e1e1e', text: '#d4d4d4', cursor: '#aeafad', selection: '#264f78', match: '#806500', chrome: '#252526', chromeText: '#cccccc', border: '#3c3c3c', borderStrong: '#4a4a4a', hover: '#2a2d2e', accent: '#007acc', input: '#3c3c3c', tabInactive: '#2d2d2d', lineBg: '#282828', muted: '#858585' },
  { id: 'sepia', name: 'Sepia', bg: '#fbf0d9', text: '#65492c', cursor: '#65492c', selection: '#e9d7b9', match: '#f3d27a', chrome: '#f3e6cc', chromeText: '#65492c', border: '#decdae', borderStrong: '#c9b591', hover: '#eadcbe', accent: '#a2602c', input: '#fff8e9', tabInactive: '#eddfc4', lineBg: '#f5e8cf', muted: '#927a5c' },
  { id: 'monokai', name: 'Monokai', bg: '#272822', text: '#f8f8f2', cursor: '#f8f8f2', selection: '#49483e', match: '#a6e22e', chrome: '#1e1f1c', chromeText: '#f8f8f2', border: '#3b3c36', borderStrong: '#4a4b44', hover: '#34352e', accent: '#a6e22e', input: '#3b3c36', tabInactive: '#292a25', lineBg: '#303127', muted: '#9d9c91' },
  { id: 'solarized-light', name: 'Solarized Light', bg: '#fdf6e3', text: '#657b83', cursor: '#657b83', selection: '#eee8d5', match: '#b58900', chrome: '#eee8d5', chromeText: '#586e75', border: '#d6cfbe', borderStrong: '#c3bba6', hover: '#e5ddc7', accent: '#268bd2', input: '#fdf6e3', tabInactive: '#e9e2cd', lineBg: '#f4eedb', muted: '#93a1a1' },
  { id: 'solarized-dark', name: 'Solarized Dark', bg: '#002b36', text: '#839496', cursor: '#93a1a1', selection: '#073642', match: '#b58900', chrome: '#073642', chromeText: '#93a1a1', border: '#14505b', borderStrong: '#1d6571', hover: '#0b3b47', accent: '#2aa198', input: '#073642', tabInactive: '#06313b', lineBg: '#05323d', muted: '#657b83' },
  { id: 'dracula', name: 'Dracula', bg: '#282a36', text: '#f8f8f2', cursor: '#f8f8f2', selection: '#44475a', match: '#6272a4', chrome: '#21222c', chromeText: '#f8f8f2', border: '#3b3d4d', borderStrong: '#4b4d61', hover: '#353746', accent: '#bd93f9', input: '#3b3d4d', tabInactive: '#2b2d3a', lineBg: '#303243', muted: '#a4a7c0' },
  { id: 'nord', name: 'Nord', bg: '#2e3440', text: '#d8dee9', cursor: '#d8dee9', selection: '#434c5e', match: '#ebcb8b', chrome: '#3b4252', chromeText: '#eceff4', border: '#4c566a', borderStrong: '#5b657a', hover: '#434c5e', accent: '#88c0d0', input: '#4c566a', tabInactive: '#3b4252', lineBg: '#353c4a', muted: '#a4afc4' },
  { id: 'github-dark', name: 'GitHub Dark', bg: '#0d1117', text: '#c9d1d9', cursor: '#c9d1d9', selection: '#1f6feb55', match: '#9e6a03', chrome: '#161b22', chromeText: '#c9d1d9', border: '#30363d', borderStrong: '#444c56', hover: '#21262d', accent: '#238636', input: '#30363d', tabInactive: '#1c2128', lineBg: '#161b22', muted: '#8b949e' },
  { id: 'one-dark', name: 'One Dark', bg: '#282c34', text: '#abb2bf', cursor: '#abb2bf', selection: '#3e4451', match: '#d19a66', chrome: '#21252b', chromeText: '#abb2bf', border: '#3b4048', borderStrong: '#4b515c', hover: '#2c323c', accent: '#61afef', input: '#3b4048', tabInactive: '#2c323c', lineBg: '#2d323b', muted: '#7f8490' },
  { id: 'gruvbox', name: 'Gruvbox Dark', bg: '#282828', text: '#ebdbb2', cursor: '#ebdbb2', selection: '#45475a', match: '#fabd2f', chrome: '#1d2021', chromeText: '#ebdbb2', border: '#3c3836', borderStrong: '#504945', hover: '#32302f', accent: '#fe8019', input: '#3c3836', tabInactive: '#282828', lineBg: '#32302f', muted: '#a89984' },
  { id: 'high-contrast', name: 'High Contrast', bg: '#000000', text: '#ffffff', cursor: '#ffffff', selection: '#0000aa', match: '#ffff00', chrome: '#000000', chromeText: '#ffffff', border: '#777777', borderStrong: '#aaaaaa', hover: '#333333', accent: '#00ff00', input: '#000000', tabInactive: '#111111', lineBg: '#1a1a1a', muted: '#cccccc' },
];

const COLOR_KEYS = ['bg', 'text', 'cursor', 'selection', 'match', 'chrome', 'chromeText', 'border', 'borderStrong', 'hover', 'accent', 'input', 'tabInactive', 'lineBg', 'muted'];

let customThemes = [];
const allThemes = () => [...BUILTIN_THEMES, ...customThemes];
const activeTheme = () => allThemes().find((theme) => theme.id === settings.themeId) || BUILTIN_THEMES[0];

const DEFAULT_KEYBINDINGS = {
  newFile: 'Ctrl+N', openFile: 'Ctrl+O', openFolder: 'Ctrl+Shift+O', save: 'Ctrl+S',
  saveAs: 'Ctrl+Shift+S', exit: 'Ctrl+Q', undo: 'Ctrl+Z', redo: 'Ctrl+Y',
  cut: 'Ctrl+X', copy: 'Ctrl+C', copyAll: 'Ctrl+Shift+C', paste: 'Ctrl+V',
  selectAll: 'Ctrl+A', find: 'Ctrl+F', replace: 'Ctrl+H', theme: 'Ctrl+T',
  zoomIn: 'Ctrl+Plus', zoomOut: 'Ctrl+Minus', toggleSide: 'Ctrl+B',
  togglePanel: 'Ctrl+Backquote', settings: 'Ctrl+Comma',
};

const DEFAULT_SETTINGS = {
  themeId: 'light', fontSize: 14,
  fontFamily: "Consolas, 'SF Mono', 'Courier New', monospace",
  tabSize: 4, insertSpaces: true, lineHeight: 21,
  showLineNumbers: true, highlightActiveLine: true, wordWrap: false,
  showWhitespace: false, cursorStyle: 'line', cursorBlink: 500,
  scrollBeyondLastLine: true, autoSave: 'off', autoSaveDelay: 1200,
  confirmUnsaved: true, recentFilesLimit: 12,
  uiDensity: 'default', showMenuBar: true, showStatusBar: true,
  showTitleFile: true, overrideAccent: false, accentColor: '#007acc',
  restoreWorkspace: true, keybindings: { ...DEFAULT_KEYBINDINGS },
};

const settings = { ...DEFAULT_SETTINGS, keybindings: { ...DEFAULT_KEYBINDINGS } };
let workspaceFolders = [];
let recentFiles = [];
const directoryCache = new Map();
const expandedDirs = new Set();
let autoSaveTimer = null;

function loadSettings() {
  try {
    const saved = JSON.parse(localStorage.getItem('pezli-settings') || '{}');
    Object.assign(settings, saved, { keybindings: { ...DEFAULT_KEYBINDINGS, ...(saved.keybindings || {}) } });
  } catch (_) { /* ignore */ }
}

function saveSettings() {
  try { localStorage.setItem('pezli-settings', JSON.stringify(settings)); } catch (_) { /* ignore */ }
}

function loadCustomThemes() {
  try { customThemes = JSON.parse(localStorage.getItem('pezli-custom-themes') || '[]'); }
  catch (_) { customThemes = []; }
}

function saveCustomThemes() {
  try { localStorage.setItem('pezli-custom-themes', JSON.stringify(customThemes)); } catch (_) { /* ignore */ }
}

function loadWorkspace() {
  try {
    workspaceFolders = JSON.parse(localStorage.getItem('pezli-workspace') || '[]');
    recentFiles = JSON.parse(localStorage.getItem('pezli-recent') || '[]');
  } catch (_) { /* ignore */ }
}

function saveWorkspace() {
  try {
    localStorage.setItem('pezli-workspace', JSON.stringify(workspaceFolders));
    localStorage.setItem('pezli-recent', JSON.stringify(recentFiles));
  } catch (_) { /* ignore */ }
}

const EXTENSIONS = ['txt', 'md', 'dart', 'js', 'html', 'css', 'json', 'xml'];
const FILE_FILTERS = [{ name: 'Text / Code Files', extensions: EXTENSIONS }];

const els = {};
let docId = 0;
let snackTimer = null;
let modalClose = null;
let charWidthCache = null;
let outputLines = [];

const layout = {
  activitySide: 'left',
  activityBarVisible: true,
  sideVisible: true,
  sideSide: 'left',
  sideWidth: 260,
  sideView: 'explorer',
  panelVisible: true,
  panelPosition: 'bottom',
  panelSize: 190,
  panelView: 'problems',
};

const state = {
  buffer: [''],
  cursorX: 0,
  cursorY: 0,
  currentFilePath: null,
  currentFileName: 'Untitled-1',
  isModified: false,
  undoStack: [],
  redoStack: [],
  selectionStartX: null,
  selectionStartY: null,
  selectionEndX: null,
  selectionEndY: null,
  findText: '',
  replaceText: '',
  findResults: [],
  currentFindIndex: -1,
};

const docs = [];
let activeId = null;

/* ---------------- Tauri file / system bridges ---------------- */

async function pickOpenFile() {
  const result = await invoke('plugin:dialog|open', {
    options: { multiple: false, title: 'Open File', filters: FILE_FILTERS },
  });
  return typeof result === 'string' ? result : null;
}

async function pickSaveFile(defaultPath) {
  const result = await invoke('plugin:dialog|save', {
    options: { title: 'Save File', defaultPath, filters: FILE_FILTERS },
  });
  return typeof result === 'string' ? result : null;
}

const readFileText = (path) => invoke('read_text_file', { path });
const writeFileText = (path, content) => invoke('write_text_file', { path, content });
const clipboardWrite = (text) => invoke('plugin:clipboard-manager|write_text', { text });
const clipboardRead = () => invoke('plugin:clipboard-manager|read_text');
const exitApp = () => invoke('plugin:process|exit', { code: 0 });

/* ---------------- Small UI helpers ---------------- */

function showSnackbar(message, color = '#252526') {
  const bar = els.snackbar;
  bar.textContent = message;
  bar.style.background = color;
  bar.classList.add('show');
  clearTimeout(snackTimer);
  snackTimer = setTimeout(() => bar.classList.remove('show'), 1800);
  logOutput(message);
}

function logOutput(message) {
  const time = new Date().toLocaleTimeString([], { hour12: false });
  outputLines.push({ time, message });
  if (outputLines.length > 300) outputLines.shift();
  if (layout.panelView === 'output' && layout.panelVisible) renderPanel();
}

function openModal({ title, icon = '', bodyHtml = '', actions = [], onMount }) {
  if (modalClose) modalClose();
  const root = els.modalRoot;
  root.innerHTML = `
    <div class="modal-overlay">
      <div class="modal" role="dialog" aria-modal="true">
        <div class="modal-header">${icon ? `<span>${icon}</span>` : ''}<span>${esc(title)}</span></div>
        <div class="modal-body">${bodyHtml}</div>
        <div class="modal-footer"></div>
      </div>
    </div>`;
  const overlay = $('.modal-overlay', root);
  const footer = $('.modal-footer', root);
  const close = () => {
    root.replaceChildren();
    if (modalClose === close) modalClose = null;
  };
  modalClose = close;
  actions.forEach((action) => {
    const btn = document.createElement('button');
    btn.className = 'btn' + (action.primary ? ' btn-primary' : '');
    btn.textContent = action.label;
    btn.addEventListener('click', () => action.onClick?.(close, root));
    footer.appendChild(btn);
  });
  overlay.addEventListener('mousedown', (event) => { if (event.target === overlay) close(); });
  onMount?.(root, close);
  return close;
}

function showContextMenu(x, y, items) {
  const menu = els.contextMenu;
  menu.replaceChildren();
  items.forEach((item) => {
    if (item.separator) {
      const sep = document.createElement('div');
      sep.className = 'sep';
      menu.appendChild(sep);
      return;
    }
    const btn = document.createElement('button');
    btn.className = 'menu-item';
    btn.innerHTML = `${iconSvg(item.icon || 'check')}<span>${esc(item.label)}</span>`;
    btn.addEventListener('click', () => { hideContextMenu(); item.action?.(); });
    menu.appendChild(btn);
  });
  menu.classList.add('show');
  menu.style.left = `${Math.min(x, window.innerWidth - 230)}px`;
  menu.style.top = `${Math.max(4, Math.min(y, window.innerHeight - menu.offsetHeight - 10))}px`;
}

function hideContextMenu() {
  els.contextMenu.classList.remove('show');
}

/* ---------------- Layout engine ---------------- */

function saveLayout() {
  try { localStorage.setItem('pezli-layout', JSON.stringify(layout)); } catch (_) { /* ignore */ }
}

function loadLayout() {
  try {
    const saved = JSON.parse(localStorage.getItem('pezli-layout') || '{}');
    Object.assign(layout, saved);
  } catch (_) { /* ignore */ }
}

function applyLayout() {
  const { workspace, activityBar, sideBar, sideSplitter, center, editorZone, panelSplitter, panel } = els;

  center.replaceChildren(editorZone);
  if (layout.panelVisible && layout.panelPosition === 'bottom') {
    panelSplitter.className = 'splitter horizontal';
    center.append(panelSplitter, panel);
    panel.style.height = `${layout.panelSize}px`;
    panel.style.width = '';
  } else {
    panelSplitter.className = 'splitter vertical';
    panel.style.height = '';
    panel.style.width = `${layout.panelSize}px`;
  }

  const order = [];
  if (layout.activityBarVisible && layout.activitySide === 'left') order.push(activityBar);
  if (layout.panelVisible && layout.panelPosition === 'left') order.push(panelSplitter, panel);
  if (layout.sideVisible && layout.sideSide === 'left') order.push(sideBar, sideSplitter);
  order.push(center);
  if (layout.sideVisible && layout.sideSide === 'right') order.push(sideSplitter, sideBar);
  if (layout.panelVisible && layout.panelPosition === 'right') order.push(panelSplitter, panel);
  if (layout.activityBarVisible && layout.activitySide === 'right') order.push(activityBar);
  workspace.replaceChildren(...order);

  workspace.className = [
    layout.activitySide === 'right' ? 'activity-right' : '',
    layout.sideSide === 'right' ? 'side-right' : '',
    layout.panelPosition === 'right' ? 'panel-right' : '',
    layout.panelPosition === 'left' ? 'panel-left' : '',
  ].filter(Boolean).join(' ');

  sideBar.style.width = `${layout.sideWidth}px`;
  sideBar.classList.toggle('hidden', !layout.sideVisible);
  panel.classList.toggle('hidden', !layout.panelVisible);
  activityBar.classList.toggle('hidden', !layout.activityBarVisible);

  $$('.activity').forEach((btn) => btn.classList.toggle('active', btn.dataset.view === layout.sideView && layout.sideVisible));
  saveLayout();
}

function setupSplitters() {
  els.sideSplitter.addEventListener('pointerdown', (event) => {
    event.preventDefault();
    const direction = layout.sideSide === 'left' ? 1 : -1;
    const startWidth = layout.sideWidth;
    const startX = event.clientX;
    document.body.classList.add('resizing');
    const move = (e) => {
      layout.sideWidth = clamp(startWidth + (e.clientX - startX) * direction, 160, 560);
      applyLayout();
    };
    const stop = () => {
      document.body.classList.remove('resizing');
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', stop);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', stop);
  });

  els.panelSplitter.addEventListener('pointerdown', (event) => {
    event.preventDefault();
    const startSize = layout.panelSize;
    const startX = event.clientX;
    const startY = event.clientY;
    document.body.classList.add('resizing');
    const move = (e) => {
      if (layout.panelPosition === 'bottom') {
        layout.panelSize = clamp(startSize + (startY - e.clientY), 90, 520);
      } else if (layout.panelPosition === 'right') {
        layout.panelSize = clamp(startSize + (startX - e.clientX), 180, 640);
      } else {
        layout.panelSize = clamp(startSize + (e.clientX - startX), 180, 640);
      }
      applyLayout();
    };
    const stop = () => {
      document.body.classList.remove('resizing');
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', stop);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', stop);
  });
}

function showDropHint(position) {
  $$('.drop-hint').forEach((el) => el.remove());
  if (!position) return;
  const rect = els.workspace.getBoundingClientRect();
  const hint = document.createElement('div');
  hint.className = 'drop-hint';
  if (position === 'left') Object.assign(hint.style, { left: `${rect.left}px`, top: `${rect.top}px`, width: '25%', height: '100%' });
  if (position === 'right') Object.assign(hint.style, { left: `${rect.right - rect.width * 0.25}px`, top: `${rect.top}px`, width: '25%', height: '100%' });
  if (position === 'bottom') Object.assign(hint.style, { left: `${rect.left}px`, top: `${rect.top + rect.height * 0.68}px`, width: '100%', height: '32%' });
  document.body.appendChild(hint);
}

function targetPosition(x, y) {
  const rect = els.workspace.getBoundingClientRect();
  if (y > rect.top + rect.height * 0.68) return 'bottom';
  if (x < rect.left + rect.width * 0.25) return 'left';
  if (x > rect.right - rect.width * 0.25) return 'right';
  return null;
}

function setupDragging() {
  els.activityBar.addEventListener('dragstart', (event) => {
    event.dataTransfer.setData('text/plain', 'activity');
    event.dataTransfer.effectAllowed = 'move';
    document.body.classList.add('dragging-activity');
  });
  els.activityBar.addEventListener('dragend', (event) => {
    document.body.classList.remove('dragging-activity');
    showDropHint(null);
    const rect = els.workspace.getBoundingClientRect();
    layout.activitySide = event.clientX < rect.left + rect.width / 2 ? 'left' : 'right';
    applyLayout();
  });

  els.panelHeader.addEventListener('dragstart', (event) => {
    event.dataTransfer.setData('text/plain', 'panel');
    event.dataTransfer.effectAllowed = 'move';
    document.body.classList.add('dragging-panel');
  });
  els.panelHeader.addEventListener('dragend', (event) => {
    document.body.classList.remove('dragging-panel');
    showDropHint(null);
    const position = targetPosition(event.clientX, event.clientY);
    if (position) {
      layout.panelPosition = position;
      layout.panelVisible = true;
      applyLayout();
      renderPanel();
    }
  });

  document.addEventListener('dragover', (event) => {
    if (document.body.classList.contains('dragging-activity')) {
      const rect = els.workspace.getBoundingClientRect();
      showDropHint(event.clientX < rect.left + rect.width / 2 ? 'left' : 'right');
    } else if (document.body.classList.contains('dragging-panel')) {
      showDropHint(targetPosition(event.clientX, event.clientY));
    }
  });
}

function layoutMenu(x, y) {
  showContextMenu(x, y, [
    { label: layout.sideVisible ? 'Hide Side Bar' : 'Show Side Bar', icon: 'sidebar', action: () => { layout.sideVisible = !layout.sideVisible; applyLayout(); } },
    { label: 'Side Bar: Left', icon: 'move', action: () => { layout.sideSide = 'left'; layout.sideVisible = true; applyLayout(); } },
    { label: 'Side Bar: Right', icon: 'move', action: () => { layout.sideSide = 'right'; layout.sideVisible = true; applyLayout(); } },
    { separator: true },
    { label: layout.panelVisible ? 'Hide Panel' : 'Show Panel', icon: 'columns', action: () => { layout.panelVisible = !layout.panelVisible; applyLayout(); } },
    { label: 'Panel: Bottom', icon: 'columns', action: () => { layout.panelPosition = 'bottom'; layout.panelVisible = true; applyLayout(); renderPanel(); } },
    { label: 'Panel: Right', icon: 'columns', action: () => { layout.panelPosition = 'right'; layout.panelVisible = true; applyLayout(); renderPanel(); } },
    { label: 'Panel: Left', icon: 'columns', action: () => { layout.panelPosition = 'left'; layout.panelVisible = true; applyLayout(); renderPanel(); } },
    { separator: true },
    { label: 'Reset Layout', icon: 'refresh-cw', action: resetLayout },
  ]);
}

function resetLayout() {
  Object.assign(layout, {
    activitySide: 'left', sideVisible: true, sideSide: 'left', sideWidth: 260,
    sideView: 'explorer', panelVisible: true, panelPosition: 'bottom',
    panelSize: 190, panelView: 'problems',
  });
  applyLayout();
  renderPanel();
  renderSide();
  showSnackbar('Layout reset');
}

/* ---------------- Document / tab model ---------------- */

function captureDoc() {
  return {
    buffer: [...state.buffer], cursorX: state.cursorX, cursorY: state.cursorY,
    currentFilePath: state.currentFilePath, currentFileName: state.currentFileName,
    isModified: state.isModified, undoStack: state.undoStack.map((s) => ({ ...s, buffer: [...s.buffer] })),
    redoStack: state.redoStack.map((s) => ({ ...s, buffer: [...s.buffer] })),
    selectionStartX: state.selectionStartX, selectionStartY: state.selectionStartY,
    selectionEndX: state.selectionEndX, selectionEndY: state.selectionEndY,
    findText: state.findText, replaceText: state.replaceText,
    findResults: state.findResults.map((r) => ({ ...r })), currentFindIndex: state.currentFindIndex,
  };
}

function restoreDoc(doc) {
  Object.assign(state, doc.buffer ? {
    buffer: [...doc.buffer], undoStack: doc.undoStack.map((s) => ({ ...s, buffer: [...s.buffer] })),
    redoStack: doc.redoStack.map((s) => ({ ...s, buffer: [...s.buffer] })),
    findResults: doc.findResults.map((r) => ({ ...r })),
  } : {});
  Object.assign(state, {
    cursorX: doc.cursorX, cursorY: doc.cursorY,
    currentFilePath: doc.currentFilePath, currentFileName: doc.currentFileName,
    isModified: doc.isModified,
    selectionStartX: doc.selectionStartX, selectionStartY: doc.selectionStartY,
    selectionEndX: doc.selectionEndX, selectionEndY: doc.selectionEndY,
    findText: doc.findText, replaceText: doc.replaceText, currentFindIndex: doc.currentFindIndex,
  });
}

function currentDoc() {
  return docs.find((doc) => doc.id === activeId);
}

function makeDoc(name, path, content = null, saved = false) {
  const doc = { id: ++docId, name, path, snapshot: null };
  if (content !== null) loadContentIntoState(content, path, name, saved);
  doc.snapshot = captureDoc();
  docs.push(doc);
  activeId = doc.id;
  renderAll();
  return doc;
}

function loadContentIntoState(content, path, name, saved = true) {
  state.buffer = content.split('\n');
  if (!state.buffer.length) state.buffer = [''];
  state.cursorX = 0;
  state.cursorY = 0;
  state.currentFilePath = path;
  state.currentFileName = name;
  state.isModified = !saved;
  state.undoStack = [];
  state.redoStack = [];
  state.findResults = [];
  state.currentFindIndex = -1;
  clearSelectionState();
  saveState();
}

function switchTab(id) {
  if (id === activeId) return;
  const doc = currentDoc();
  if (doc) doc.snapshot = captureDoc();
  activeId = id;
  const next = currentDoc();
  if (next && next.type !== 'settings') restoreDoc(next.snapshot || {});
  charWidthCache = null;
  renderAll();
}

function closeTab(id) {
  const doc = docs.find((d) => d.id === id);
  if (!doc) return;
  if (doc.id === activeId) {
    if (state.isModified) {
      showUnsavedDialog(async () => { doCloseTab(doc); });
      return;
    }
  }
  doCloseTab(doc);
}

function doCloseTab(doc) {
  if (doc.id === activeId) {
    const docSnap = currentDoc();
    if (docSnap) docSnap.snapshot = captureDoc();
  }
  const index = docs.indexOf(doc);
  if (index >= 0) docs.splice(index, 1);
  if (!docs.length) {
    makeDoc('Untitled-1');
    return;
  }
  if (doc.id === activeId) {
    activeId = docs[Math.max(0, index - 1)].id;
    if (currentDoc()?.type !== 'settings') restoreDoc(currentDoc().snapshot || {});
  }
  renderAll();
}

function refreshActiveTab() {
  const doc = currentDoc();
  if (!doc || doc.type === 'settings') return;
  doc.name = state.currentFileName;
  doc.path = state.currentFilePath;
  doc.snapshot = captureDoc();
  renderTabs();
  renderSide();
  updateChrome();
  if (settings.autoSave === 'delay' && state.isModified) {
    clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(() => saveFile(), settings.autoSaveDelay);
  }
}

/* ---------------- Editor state helpers ---------------- */

function fileContent() { return state.buffer.join('\n'); }

function snapshotState() {
  return { buffer: [...state.buffer], cursorX: state.cursorX, cursorY: state.cursorY };
}

function saveState() {
  state.undoStack.push(snapshotState());
  if (state.undoStack.length > 100) state.undoStack.shift();
  state.redoStack.length = 0;
}

function clearSelectionState() {
  state.selectionStartX = state.selectionStartY = null;
  state.selectionEndX = state.selectionEndY = null;
}

function startSelection() {
  state.selectionStartX = state.cursorX;
  state.selectionStartY = state.cursorY;
}

function updateSelection() {
  state.selectionEndX = state.cursorX;
  state.selectionEndY = state.cursorY;
}

function hasSelection() {
  return state.selectionStartX !== null && state.selectionStartY !== null &&
    state.selectionEndX !== null && state.selectionEndY !== null &&
    (state.selectionStartX !== state.selectionEndX || state.selectionStartY !== state.selectionEndY);
}

function normalizedSelection() {
  let [sy, sx, ey, ex] = [state.selectionStartY, state.selectionStartX, state.selectionEndY, state.selectionEndX];
  if (sy > ey || (sy === ey && sx > ex)) [sy, sx, ey, ex] = [ey, ex, sy, sx];
  return { sy, sx, ey, ex };
}

function getSelectedText() {
  if (!hasSelection()) return '';
  const { sy, sx, ey, ex } = normalizedSelection();
  if (sy === ey) return state.buffer[sy].slice(sx, ex);
  let result = state.buffer[sy].slice(sx) + '\n';
  for (let i = sy + 1; i < ey; i++) result += state.buffer[i] + '\n';
  return result + state.buffer[ey].slice(0, ex);
}

function deleteSelection() {
  if (!hasSelection()) return;
  const { sy, sx, ey, ex } = normalizedSelection();
  if (sy === ey) {
    const line = state.buffer[sy];
    state.buffer[sy] = line.slice(0, sx) + line.slice(ex);
  } else {
    const first = state.buffer[sy].slice(0, sx);
    const last = state.buffer[ey].slice(ex);
    state.buffer[sy] = first + last;
    for (let i = ey; i > sy; i--) state.buffer.splice(i, 1);
  }
  state.cursorX = sx;
  state.cursorY = sy;
  clearSelectionState();
}

function selectAll() {
  state.selectionStartX = 0;
  state.selectionStartY = 0;
  state.selectionEndY = state.buffer.length - 1;
  state.selectionEndX = state.buffer[state.buffer.length - 1].length;
  render();
}

/* ---------------- Editing ---------------- */

function insertCharacter(ch) {
  saveState();
  deleteSelection();
  state.isModified = true;
  const line = state.buffer[state.cursorY] ?? '';
  const padded = line.length < state.cursorX ? line + ' '.repeat(state.cursorX - line.length) : line;
  state.buffer[state.cursorY] = padded.slice(0, state.cursorX) + ch + padded.slice(state.cursorX);
  state.cursorX++;
  render();
  refreshActiveTab();
}

function handleBackspace() {
  saveState();
  if (hasSelection()) deleteSelection();
  else if (state.cursorX > 0) {
    const line = state.buffer[state.cursorY];
    state.buffer[state.cursorY] = line.slice(0, state.cursorX - 1) + line.slice(state.cursorX);
    state.cursorX--;
  } else if (state.cursorY > 0) {
    state.cursorX = state.buffer[state.cursorY - 1].length;
    state.buffer[state.cursorY - 1] += state.buffer[state.cursorY];
    state.buffer.splice(state.cursorY, 1);
    state.cursorY--;
  }
  state.isModified = true;
  render();
  refreshActiveTab();
}

function handleDelete() {
  saveState();
  if (hasSelection()) deleteSelection();
  else {
    const line = state.buffer[state.cursorY];
    if (state.cursorX < line.length) {
      state.buffer[state.cursorY] = line.slice(0, state.cursorX) + line.slice(state.cursorX + 1);
    } else if (state.cursorY < state.buffer.length - 1) {
      state.buffer[state.cursorY] += state.buffer[state.cursorY + 1];
      state.buffer.splice(state.cursorY + 1, 1);
    }
  }
  state.isModified = true;
  render();
  refreshActiveTab();
}

function handleEnter() {
  saveState();
  deleteSelection();
  const line = state.buffer[state.cursorY];
  state.buffer[state.cursorY] = line.slice(0, state.cursorX);
  state.buffer.splice(state.cursorY + 1, 0, line.slice(state.cursorX));
  state.cursorY++;
  state.cursorX = 0;
  state.isModified = true;
  render();
  refreshActiveTab();
}

function handleTab() {
  const indent = settings.insertSpaces ? ' '.repeat(settings.tabSize) : '\t';
  if (hasSelection()) {
    saveState();
    const { sy, sx, ex } = normalizedSelection();
    const endY = state.selectionEndY;
    for (let i = sy; i <= endY; i++) state.buffer[i] = indent + state.buffer[i];
    state.selectionStartX = sx + indent.length;
    state.selectionEndX = ex + indent.length;
    state.isModified = true;
    render();
    refreshActiveTab();
    return;
  }
  insertCharacter(indent);
}

function moveCursor(dx, dy, extend = false) {
  if (extend && !hasSelection()) startSelection();
  else if (!extend) clearSelectionState();
  if (dx !== 0) {
    const newX = state.cursorX + dx;
    if (newX < 0 && state.cursorY > 0) {
      state.cursorY--;
      state.cursorX = state.buffer[state.cursorY].length;
    } else if (newX > state.buffer[state.cursorY].length && state.cursorY < state.buffer.length - 1) {
      state.cursorY++;
      state.cursorX = 0;
    } else {
      state.cursorX = clamp(newX, 0, state.buffer[state.cursorY].length);
    }
  }
  if (dy !== 0) {
    state.cursorY = clamp(state.cursorY + dy, 0, state.buffer.length - 1);
    state.cursorX = clamp(state.cursorX, 0, state.buffer[state.cursorY].length);
  }
  if (extend) updateSelection();
  render();
  refreshActiveTab();
}

function moveToLineStart(extend = false) {
  if (extend && !hasSelection()) startSelection();
  else if (!extend) clearSelectionState();
  state.cursorX = 0;
  if (extend) updateSelection();
  render();
}

function moveToLineEnd(extend = false) {
  if (extend && !hasSelection()) startSelection();
  else if (!extend) clearSelectionState();
  state.cursorX = state.buffer[state.cursorY].length;
  if (extend) updateSelection();
  render();
}

function undo() {
  if (state.undoStack.length <= 1) return;
  state.redoStack.push(snapshotState());
  state.undoStack.pop();
  const prev = state.undoStack[state.undoStack.length - 1];
  state.buffer = [...prev.buffer];
  state.cursorX = prev.cursorX;
  state.cursorY = prev.cursorY;
  state.isModified = state.undoStack.length > 1;
  clearSelectionState();
  render();
  refreshActiveTab();
  showSnackbar('Undone');
}

function redo() {
  if (!state.redoStack.length) return;
  const next = state.redoStack.pop();
  state.undoStack.push(snapshotState());
  state.buffer = [...next.buffer];
  state.cursorX = next.cursorX;
  state.cursorY = next.cursorY;
  state.isModified = true;
  clearSelectionState();
  render();
  refreshActiveTab();
  showSnackbar('Redone');
}

async function cut() {
  if (!hasSelection()) return;
  try { await clipboardWrite(getSelectedText()); } catch (_) { /* ignore */ }
  saveState();
  deleteSelection();
  state.isModified = true;
  render();
  refreshActiveTab();
  showSnackbar('Text cut to clipboard');
}

async function copy() {
  if (!hasSelection()) return;
  try { await clipboardWrite(getSelectedText()); showSnackbar('Text copied to clipboard'); }
  catch (error) { showErrorDialog('Clipboard Error', error.message); }
}

async function paste() {
  let text;
  try { text = await clipboardRead(); }
  catch (error) { showErrorDialog('Clipboard Error', error.message); return; }
  if (text === null || text === undefined) return;
  saveState();
  deleteSelection();
  const lines = text.split('\n');
  state.isModified = true;
  if (lines.length === 1) {
    const line = state.buffer[state.cursorY];
    state.buffer[state.cursorY] = line.slice(0, state.cursorX) + lines[0] + line.slice(state.cursorX);
    state.cursorX += lines[0].length;
  } else {
    const line = state.buffer[state.cursorY];
    const before = line.slice(0, state.cursorX);
    const after = line.slice(state.cursorX);
    state.buffer[state.cursorY] = before + lines[0];
    for (let i = 1; i < lines.length - 1; i++) state.buffer.splice(state.cursorY + i, 0, lines[i]);
    state.buffer.splice(state.cursorY + lines.length - 1, 0, lines[lines.length - 1] + after);
    state.cursorY += lines.length - 1;
    state.cursorX = lines[lines.length - 1].length;
  }
  render();
  refreshActiveTab();
  showSnackbar('Text pasted');
}

async function copyAll() {
  if (!fileContent()) { showSnackbar('No content to copy', '#c98614'); return; }
  try { await clipboardWrite(fileContent()); showSnackbar('All content copied to clipboard', '#16825d'); }
  catch (error) { showErrorDialog('Clipboard Error', error.message); }
}

/* ---------------- Find / replace ---------------- */

function performFind() {
  if (!state.findText) return;
  state.findResults = [];
  state.currentFindIndex = -1;
  state.buffer.forEach((line, lineIndex) => {
    let index = 0;
    while (true) {
      const found = line.indexOf(state.findText, index);
      if (found === -1) break;
      state.findResults.push({ line: lineIndex, start: found, end: found + state.findText.length });
      index = found + 1;
    }
  });
  if (state.findResults.length) {
    state.currentFindIndex = 0;
    goToFindResult(0);
    showSnackbar(`Found ${state.findResults.length} matches`);
  } else {
    showSnackbar('No matches found', '#c98614');
  }
  renderSearchViews();
}

function findNext(backwards = false) {
  if (!state.findText) { showFindDialog(); return; }
  if (!state.findResults.length) { performFind(); return; }
  const count = state.findResults.length;
  state.currentFindIndex = (state.currentFindIndex + (backwards ? -1 : 1) + count) % count;
  goToFindResult(state.currentFindIndex);
  renderSearchViews();
}

function performReplaceAll() {
  if (!state.findText) return;
  saveState();
  let replacements = 0;
  state.buffer = state.buffer.map((line) => {
    if (!line.includes(state.findText)) return line;
    replacements += line.split(state.findText).length - 1;
    return line.split(state.findText).join(state.replaceText);
  });
  state.isModified = true;
  state.findResults = [];
  render();
  refreshActiveTab();
  renderSearchViews();
  showSnackbar(`Replaced ${replacements} occurrences`, '#16825d');
}

function goToFindResult(index) {
  const result = state.findResults[index];
  if (!result) return;
  state.cursorY = result.line;
  state.cursorX = result.start;
  state.selectionStartY = result.line;
  state.selectionStartX = result.start;
  state.selectionEndY = result.line;
  state.selectionEndX = result.end;
  render();
  const row = $(`.line[data-line="${result.line}"]`, els.editor);
  row?.scrollIntoView({ block: 'center', behavior: 'smooth' });
  refreshActiveTab();
}

function renderSearchViews() {
  renderSide();
  renderPanel();
}

/* ---------------- File operations ---------------- */

async function newFile() {
  if (state.isModified) {
    showUnsavedDialog(() => makeDoc(`Untitled-${docId + 1}`, null));
  } else {
    makeDoc(`Untitled-${docId + 1}`, null);
  }
}

async function openFile() {
  try {
    const path = await pickOpenFile();
    if (!path) return;
    const content = await readFileText(path);
    const process = () => {
      const existing = docs.find((doc) => doc.path === path);
      if (existing) { switchTab(existing.id); showSnackbar(`Opened: ${baseName(path)}`, '#16825d'); return; }
      makeDoc(baseName(path), path, content, true);
      showSnackbar(`Opened: ${baseName(path)}`, '#16825d');
    };
    if (state.isModified) showUnsavedDialog(process);
    else process();
  } catch (error) {
    showErrorDialog('Open File Error', error.message || String(error));
  }
}

async function saveFile() {
  try {
    let path = state.currentFilePath;
    if (!path) path = await pickSaveFile(state.currentFileName || 'untitled.txt');
    if (!path) return;
    await writeFileText(path, fileContent());
    state.currentFilePath = path;
    state.currentFileName = baseName(path);
    state.isModified = false;
    refreshActiveTab();
    showSnackbar(`Saved: ${baseName(path)}`, '#16825d');
  } catch (error) {
    showErrorDialog('Save Error', error.message || String(error));
  }
}

async function saveAsFile() {
  try {
    const path = await pickSaveFile(state.currentFileName || 'untitled.txt');
    if (!path) return;
    await writeFileText(path, fileContent());
    state.currentFilePath = path;
    state.currentFileName = baseName(path);
    state.isModified = false;
    refreshActiveTab();
    showSnackbar(`Saved: ${baseName(path)}`, '#16825d');
  } catch (error) {
    showErrorDialog('Save Error', error.message || String(error));
  }
}

async function exitEditor() {
  const quit = async () => {
    try { await exitApp(); } catch (error) { showErrorDialog('Exit Error', error.message); }
  };
  if (state.isModified) showUnsavedDialog(quit);
  else quit();
}

/* ---------------- Dialogs ---------------- */

function showErrorDialog(title, message) {
  openModal({
    title,
    icon: iconSvg('alert-triangle'),
    bodyHtml: `<p>${esc(message)}</p>`,
    actions: [{ label: 'OK', primary: true, onClick: (close) => close() }],
  });
}

function showUnsavedDialog(onProceed) {
  openModal({
    title: 'Unsaved Changes',
    icon: iconSvg('alert-triangle'),
    bodyHtml: `<p><strong>${esc(state.currentFileName)}</strong> has unsaved changes. Save before continuing?</p>`,
    actions: [
      { label: "Don't Save", onClick: (close) => { close(); onProceed(); } },
      { label: 'Cancel', onClick: (close) => close() },
      { label: 'Save', primary: true, onClick: async (close) => { close(); await saveFile(); onProceed(); } },
    ],
  });
}

function showFindDialog() {
  openModal({
    title: 'Find',
    icon: iconSvg('search'),
    bodyHtml: `<div class="modal-field"><label for="findInput">Find in ${esc(state.currentFileName)}</label><input id="findInput" value="${esc(state.findText)}" placeholder="Search text..." /></div>`,
    actions: [
      { label: 'Cancel', onClick: (close) => close() },
      { label: 'Find Next', onClick: (close, root) => { state.findText = $('#findInput', root).value; findNext(); close(); } },
      { label: 'Find All', primary: true, onClick: (close, root) => { state.findText = $('#findInput', root).value; performFind(); close(); } },
    ],
    onMount: (root) => { const input = $('#findInput', root); input.focus(); input.select(); },
  });
}

function showReplaceDialog() {
  openModal({
    title: 'Find & Replace',
    icon: iconSvg('refresh-cw'),
    bodyHtml: `
      <div class="modal-field"><label for="replaceFind">Find</label><input id="replaceFind" value="${esc(state.findText)}" placeholder="Find..." /></div>
      <div class="modal-field"><label for="replaceWith">Replace with</label><input id="replaceWith" value="${esc(state.replaceText)}" placeholder="Replace..." /></div>`,
    actions: [
      { label: 'Cancel', onClick: (close) => close() },
      { label: 'Find', onClick: (close, root) => { state.findText = $('#replaceFind', root).value; state.replaceText = $('#replaceWith', root).value; performFind(); close(); } },
      { label: 'Replace All', primary: true, onClick: (close, root) => { state.findText = $('#replaceFind', root).value; state.replaceText = $('#replaceWith', root).value; performReplaceAll(); close(); } },
    ],
    onMount: (root) => $('#replaceFind', root).focus(),
  });
}

function showAbout() {
  openModal({
    title: 'About Pezli Editor',
    icon: iconSvg('info'),
    bodyHtml: `
      <p><strong>Pezli Editor v1.0.0</strong></p>
      <p>A keyboard-driven desktop text editor with a rearrangeable workspace, built with Tauri.</p>
      <div class="section-title">Features</div>
      <ul>
        <li>Drag and dock panels and side bars like Visual Studio Code</li>
        <li>Multiple themes, zoom, line numbers and cursor tracking</li>
        <li>Selection, clipboard, undo/redo and find &amp; replace</li>
        <li>Native open/save dialogs</li>
      </ul>
      <p class="muted">Built with patience by Beauttah Kipruto, MUST.</p>`,
    actions: [{ label: 'OK', primary: true, onClick: (close) => close() }],
  });
}

function showDeveloperInfo() {
  openModal({
    title: 'Developer Info',
    icon: iconSvg('user'),
    bodyHtml: `
      <p><strong>Beauttah Kipruto</strong></p>
      <p>Student at Meru University of Science and Technology (MUST)</p>
      <p>Software Developer</p>
      <p class="muted">Created Pezli Editor as a modern, feature-rich text editor.</p>`,
    actions: [{ label: 'Close', primary: true, onClick: (close) => close() }],
  });
}

function showShortcuts() {
  const rows = [
    ['File', ''], ['Ctrl+N', 'New file'], ['Ctrl+O', 'Open file'], ['Ctrl+S', 'Save'],
    ['Ctrl+Shift+S', 'Save as'], ['Ctrl+Q', 'Exit'],
    ['Editing', ''], ['Ctrl+Z / Ctrl+Y', 'Undo / redo'], ['Ctrl+X / C / V', 'Cut / copy / paste'],
    ['Ctrl+A', 'Select all'], ['Tab', 'Insert 4 spaces'],
    ['Workspace', ''], ['Ctrl+B', 'Toggle side bar'], ['Ctrl+`', 'Toggle panel'],
    ['Drag bars', 'Dock panels anywhere'], ['Drag tabs', 'Reorder editor tabs'],
    ['Search & View', ''], ['Ctrl+F / Ctrl+H', 'Find / replace'], ['F3 / Shift+F3', 'Next / previous match'],
    ['Ctrl+T', 'Switch theme'], ['Ctrl++ / Ctrl+-', 'Zoom in / out'],
  ];
  const grid = rows.map(([key, label]) => label === ''
    ? `<div class="section-title" style="grid-column:1/-1">${key}</div>`
    : `<span class="key">${key}</span><span>${label}</span>`).join('');
  openModal({
    title: 'Keyboard Shortcuts',
    icon: iconSvg('layout'),
    bodyHtml: `<div class="shortcut-grid">${grid}</div>`,
    actions: [{ label: 'Close', primary: true, onClick: (close) => close() }],
  });
}

/* ---------------- Rendering ---------------- */

function applyTheme() {
  const theme = activeTheme();
  const accent = settings.overrideAccent ? settings.accentColor : theme.accent;
  const style = document.documentElement.style;
  const vars = {
    '--bg': theme.bg, '--text': theme.text, '--cursor': theme.cursor,
    '--selection': theme.selection, '--match': theme.match, '--chrome': theme.chrome,
    '--chrome-text': theme.chromeText, '--border': theme.border,
    '--border-strong': theme.borderStrong, '--hover': theme.hover,
    '--accent': accent, '--input': theme.input,
    '--tab-inactive': theme.tabInactive, '--line-bg': theme.lineBg, '--muted': theme.muted,
    '--font-size': `${settings.fontSize}px`, '--line-height': `${settings.lineHeight}px`,
    '--editor-font': settings.fontFamily, '--cursor-blink': `${settings.cursorBlink}ms`,
  };
  Object.entries(vars).forEach(([key, value]) => style.setProperty(key, value));
  document.body.classList.remove('density-compact', 'density-default', 'density-spacious');
  document.body.classList.add(`density-${settings.uiDensity}`);
  $('#menubar')?.classList.toggle('hidden', !settings.showMenuBar);
  $('#statusbar')?.classList.toggle('hidden', !settings.showStatusBar);
  $('#title-file')?.classList.toggle('hidden', !settings.showTitleFile);
}

function isLineSelected(lineIndex) {
  if (!hasSelection()) return false;
  const { sy, ey } = normalizedSelection();
  return lineIndex >= sy && lineIndex <= ey;
}

function selectionRangeForLine(lineIndex, text) {
  const { sy, sx, ey, ex } = normalizedSelection();
  let start = 0;
  let end = 0;
  if (lineIndex === sy && lineIndex === ey) { start = sx; end = ex; }
  else if (lineIndex === sy) { start = sx; end = text.length; }
  else if (lineIndex === ey) { start = 0; end = ex; }
  else if (lineIndex > sy && lineIndex < ey) { start = 0; end = text.length; }
  else return null;
  return [clamp(start, 0, text.length), clamp(end, 0, text.length)];
}

function selectionSpans(text, [start, end]) {
  return esc(text.slice(0, start)) + `<span class="selection">${esc(text.slice(start, end))}</span>` + esc(text.slice(end));
}

function visualize(text) {
  const safe = esc(text);
  return settings.showWhitespace ? safe.replace(/ /g, '·').replace(/\t/g, '→') : safe;
}

function cursorSpans(text) {
  const cx = Math.min(state.cursorX, text.length);
  const before = visualize(text.slice(0, cx));
  const ch = cx < text.length ? visualize(text[cx]) : ' ';
  const after = cx < text.length - 1 ? visualize(text.slice(cx + 1)) : '';
  const cursorClass = `cursor${settings.cursorStyle !== 'line' ? ` ${settings.cursorStyle}` : ''}`;
  return `${before}<span class="${cursorClass}" id="cursor">${ch}</span>${after}`;
}

function plainSpans(text) {
  if (!state.findText) return visualize(text.length ? text : ' ');
  let result = '';
  let index = 0;
  while (true) {
    const found = text.indexOf(state.findText, index);
    if (found === -1) { result += visualize(text.slice(index)); break; }
    result += visualize(text.slice(index, found)) + `<span class="match">${visualize(state.findText)}</span>`;
    index = found + state.findText.length;
  }
  return result || ' ';
}

function buildLineHtml(text, lineIndex) {
  const selected = hasSelection() && isLineSelected(lineIndex);
  const range = selected ? selectionRangeForLine(lineIndex, text) : null;
  if (lineIndex === state.cursorY) {
    if (selected && range) return selectionSpans(text, range);
    return cursorSpans(text);
  }
  if (selected && range) return selectionSpans(text, range);
  return plainSpans(text);
}

function render() {
  applyTheme();
  if (currentDoc()?.type === 'settings') {
    renderSettingsView();
    updateChrome();
    return;
  }
  const editor = els.editor;
  const scrollTop = editor.scrollTop;
  const scrollLeft = editor.scrollLeft;
  const frag = document.createDocumentFragment();

  editor.classList.toggle('wrap', settings.wordWrap);
  editor.classList.toggle('no-highlight', !settings.highlightActiveLine);

  state.buffer.forEach((line, index) => {
    const row = document.createElement('div');
    row.className = 'line' + (index === state.cursorY ? ' active' : '');
    row.dataset.line = String(index);
    if (settings.showLineNumbers) {
      const number = document.createElement('div');
      number.className = 'line-number';
      number.textContent = String(index + 1);
      row.appendChild(number);
    }
    const content = document.createElement('div');
    content.className = 'line-content';
    content.innerHTML = buildLineHtml(line, index);
    row.appendChild(content);
    frag.appendChild(row);
  });
  const tail = document.createElement('div');
  tail.style.height = settings.scrollBeyondLastLine ? '40vh' : '8px';
  frag.appendChild(tail);

  editor.replaceChildren(frag);
  editor.scrollTop = scrollTop;
  editor.scrollLeft = scrollLeft;
  updateChrome();
}

function updateChrome() {
  const doc = currentDoc();
  const isSettings = doc?.type === 'settings';
  $('#title-file').textContent = isSettings
    ? 'Settings — Pezli Editor'
    : `${state.currentFileName}${state.isModified ? ' •' : ''} — Pezli Editor`;
  $('#status-cursor').textContent = isSettings ? '' : `Ln ${state.cursorY + 1}, Col ${state.cursorX + 1}`;
  const sel = $('#status-selection');
  sel.textContent = !isSettings && hasSelection() ? `${getSelectedText().length} selected` : '';
  const find = $('#status-find');
  find.textContent = state.findResults.length
    ? `Find: ${state.currentFindIndex + 1}/${state.findResults.length}` : '';
  $('#status-font').textContent = isSettings
    ? 'UTF-8'
    : `Spaces: ${settings.tabSize}   UTF-8   LF   ${settings.fontSize}px`;
  $('#status-theme').textContent = activeTheme().name;
  if (doc && !isSettings) doc.name = state.currentFileName;
}

function renderTabs() {
  const tabs = els.tabs;
  tabs.replaceChildren();
  docs.forEach((doc) => {
    const active = doc.id === activeId;
    const tab = document.createElement('div');
    tab.className = 'tab' + (active ? ' active' : '');
    tab.draggable = true;
    tab.dataset.id = String(doc.id);
    const name = doc.snapshot?.currentFileName || doc.name;
    const dirty = doc.snapshot?.isModified || false;
    tab.innerHTML = `
      <span class="tab-icon">${iconSvg('file', 14)}</span>
      <span class="tab-name">${esc(name)}</span>
      <span class="tab-close" data-close="${doc.id}">${dirty ? '&#8226;' : iconSvg('x', 12)}</span>`;
    tabs.appendChild(tab);
  });
  hydrateIcons(tabs);
  setupTabDnD();
}

function setupTabDnD() {
  $$('.tab', els.tabs).forEach((tab) => {
    tab.addEventListener('dragstart', (event) => {
      event.dataTransfer.setData('text/plain', tab.dataset.id);
      event.dataTransfer.effectAllowed = 'move';
    });
    tab.addEventListener('dragover', (event) => { event.preventDefault(); tab.classList.add('drag-over'); });
    tab.addEventListener('dragleave', () => tab.classList.remove('drag-over'));
    tab.addEventListener('drop', (event) => {
      event.preventDefault();
      tab.classList.remove('drag-over');
      const sourceId = Number(event.dataTransfer.getData('text/plain'));
      const targetId = Number(tab.dataset.id);
      const from = docs.findIndex((d) => d.id === sourceId);
      const to = docs.findIndex((d) => d.id === targetId);
      if (from < 0 || to < 0 || from === to) return;
      const [moved] = docs.splice(from, 1);
      docs.splice(to, 0, moved);
      renderTabs();
    });
  });
}

function renderSide() {
  const body = els.sideBody;
  body.replaceChildren();

  if (layout.sideView === 'explorer') {
    els.sideTitle.textContent = 'EXPLORER';
    const open = section('OPEN EDITORS', 'chevron-down', () => {
      const list = document.createElement('div');
      list.className = 'side-list';
      docs.forEach((doc) => {
        const row = document.createElement('button');
        row.className = 'side-row' + (doc.id === activeId ? ' active' : '');
        const dirty = doc.snapshot?.isModified ? '<span class="dirty">&#8226;</span>' : '';
        row.innerHTML = `<span class="row-icon">${iconSvg('file', 14)}</span><span class="row-name">${esc(doc.snapshot?.currentFileName || doc.name)}</span>${dirty}`;
        row.addEventListener('click', () => switchTab(doc.id));
        row.addEventListener('contextmenu', (event) => {
          event.preventDefault();
          showContextMenu(event.clientX, event.clientY, [
            { label: 'Close', icon: 'x', action: () => closeTab(doc.id) },
          ]);
        });
        list.appendChild(row);
      });
      return list;
    });
    body.appendChild(open);

    const folders = renderFoldersSection();
    if (folders) body.appendChild(folders);
    const recent = renderRecentSection();
    if (recent) body.appendChild(recent);

    const outline = section('OUTLINE', 'chevron-down', () => {
      const list = document.createElement('div');
      list.className = 'side-list';
      const pattern = /^\s*(class|def|function|interface|struct|enum|import|from|#include|public|private|fn)\b/;
      let count = 0;
      state.buffer.forEach((line, index) => {
        if (count++ > 80) return;
        const match = line.match(pattern);
        if (!match) return;
        const row = document.createElement('button');
        row.className = 'result-row';
        row.innerHTML = `<span class="ln">${index + 1}</span><span>${esc(line.trim().slice(0, 50))}</span>`;
        row.addEventListener('click', () => { state.cursorY = index; state.cursorX = 0; render(); });
        list.appendChild(row);
      });
      if (!list.children.length) {
        const empty = document.createElement('div');
        empty.className = 'side-empty';
        empty.textContent = 'No symbols found in this file.';
        list.appendChild(empty);
      }
      return list;
    });
    body.appendChild(outline);

    const actions = section('QUICK ACTIONS', 'chevron-down', () => {
      const list = document.createElement('div');
      list.className = 'side-list';
      [
        ['New File', 'file-plus', newFile], ['Open File...', 'folder', openFile],
        ['Save', 'save', saveFile], ['Find', 'search', showFindDialog],
        ['Keyboard Shortcuts', 'layout', showShortcuts],
      ].forEach(([label, iconName, action]) => {
        const row = document.createElement('button');
        row.className = 'side-row';
        row.innerHTML = `<span class="row-icon">${iconSvg(iconName, 14)}</span><span class="row-name">${label}</span>`;
        row.addEventListener('click', action);
        list.appendChild(row);
      });
      return list;
    });
    body.appendChild(actions);
  } else if (layout.sideView === 'search') {
    els.sideTitle.textContent = 'SEARCH';
    const wrap = document.createElement('div');
    wrap.className = 'side-search';
    wrap.innerHTML = `
      <input id="sideFind" placeholder="Find" value="${esc(state.findText)}" />
      <input id="sideReplace" placeholder="Replace" value="${esc(state.replaceText)}" style="margin-top:6px" />
      <div style="display:flex;gap:6px;margin-top:8px">
        <button class="btn" data-side-action="find">Find</button>
        <button class="btn" data-side-action="replace">Replace All</button>
      </div>`;
    body.appendChild(wrap);
    hydrateIcons(wrap);
    $('[data-side-action="find"]', wrap).addEventListener('click', () => {
      state.findText = $('#sideFind', wrap).value;
      performFind();
    });
    $('[data-side-action="replace"]', wrap).addEventListener('click', () => {
      state.findText = $('#sideFind', wrap).value;
      state.replaceText = $('#sideReplace', wrap).value;
      performReplaceAll();
    });
    const list = document.createElement('div');
    list.className = 'side-list';
    state.findResults.forEach((result, index) => {
      const row = document.createElement('button');
      row.className = 'result-row';
      row.innerHTML = `<span class="ln">${result.line + 1}</span><span>${esc(state.buffer[result.line].trim().slice(0, 48))}</span>`;
      row.addEventListener('click', () => { state.currentFindIndex = index; goToFindResult(index); });
      list.appendChild(row);
    });
    if (!state.findResults.length) {
      const empty = document.createElement('div');
      empty.className = 'side-empty';
      empty.textContent = state.findText ? 'No results.' : 'Enter text to search across the file.';
      list.appendChild(empty);
    }
    body.appendChild(list);
  } else {
    els.sideTitle.textContent = 'SOURCE CONTROL';
    const empty = document.createElement('div');
    empty.className = 'side-empty';
    const modified = docs.filter((doc) => doc.snapshot?.isModified).length;
    empty.textContent = modified
      ? `${modified} file(s) with unsaved changes.`
      : 'No changes. All open files are saved.';
    body.appendChild(empty);
  }
  hydrateIcons(body);
}

function section(title, iconName, contentFactory) {
  const sectionEl = document.createElement('div');
  sectionEl.className = 'side-section';
  const heading = document.createElement('button');
  heading.className = 'side-section-title';
  heading.innerHTML = `<span class="chev">${iconSvg(iconName, 14)}</span><span>${title}</span>`;
  const content = contentFactory();
  heading.addEventListener('click', () => sectionEl.classList.toggle('collapsed'));
  sectionEl.append(heading, content);
  return sectionEl;
}

function renderPanel() {
  const body = els.panelBody;
  body.replaceChildren();
  $$('.panel-tab').forEach((tab) => tab.classList.toggle('active', tab.dataset.panel === layout.panelView));

  if (layout.panelView === 'problems') {
    const dirty = docs.filter((doc) => doc.snapshot?.isModified);
    if (!dirty.length) {
      const ok = document.createElement('div');
      ok.className = 'problem-ok';
      ok.innerHTML = `${iconSvg('check-circle', 15)}<span>No problems detected in ${docs.length} open file(s).</span>`;
      body.appendChild(ok);
    } else {
      dirty.forEach((doc) => {
        const row = document.createElement('button');
        row.className = 'result-row';
        row.innerHTML = `${iconSvg('alert-triangle', 13)}<span>Unsaved changes in ${esc(doc.snapshot?.currentFileName || doc.name)}</span>`;
        row.addEventListener('click', () => switchTab(doc.id));
        body.appendChild(row);
      });
    }
  } else if (layout.panelView === 'output') {
    outputLines.slice(-80).forEach((line) => {
      const row = document.createElement('div');
      row.className = 'output-line';
      row.innerHTML = `<span class="ts">[${esc(line.time)}]</span>${esc(line.message)}`;
      body.appendChild(row);
    });
    if (!outputLines.length) body.textContent = 'Output is empty.';
  } else {
    state.findResults.forEach((result, index) => {
      const row = document.createElement('button');
      row.className = 'result-row';
      row.innerHTML = `<span class="ln">${result.line + 1}</span><span>${esc(state.buffer[result.line].trim().slice(0, 80))}</span>`;
      row.addEventListener('click', () => { state.currentFindIndex = index; goToFindResult(index); });
      body.appendChild(row);
    });
    if (!state.findResults.length) body.textContent = state.findText ? 'No results.' : 'Run Find to see results here.';
  }
  hydrateIcons(body);
}

function renderAll() {
  renderTabs();
  renderSide();
  renderPanel();
  render();
  applyLayout();
}

/* ---------------- Input handling ---------------- */

function closeMenus() {
  $$('.menu.open').forEach((menu) => menu.classList.remove('open'));
  hideContextMenu();
}

function handleAction(action, event) {
  const actions = {
    new: newFile, newFile, open: openFile, openFile, openFolder, addFolder,
    save: saveFile, saveFile, saveas: saveAsFile, saveAs: saveAsFile, share: copyAll,
    exit: exitEditor, undo, redo, cut, copy, copyAll, paste, selectall: selectAll,
    find: showFindDialog, replace: showReplaceDialog, theme: switchTheme,
    zoomin: zoomIn, zoomout: zoomOut, about: showAbout,
    developer: showDeveloperInfo, shortcuts: showShortcuts, settings: openSettings,
    toggleSide: () => { layout.sideVisible = !layout.sideVisible; applyLayout(); },
    togglePanel: () => { layout.panelVisible = !layout.panelVisible; applyLayout(); },
    layoutMenu: () => layoutMenu(event.clientX, event.clientY),
    panelMenu: () => layoutMenu(event.clientX, event.clientY),
  };
  actions[action]?.();
}

let captureBinding = null;

function comboFromEvent(event) {
  const parts = [];
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.metaKey) parts.push('Cmd');
  if (event.altKey) parts.push('Alt');
  if (event.shiftKey) parts.push('Shift');
  const namedKeys = { '+': 'Plus', '=': 'Plus', '-': 'Minus', '`': 'Backquote', ',': 'Comma', '.': 'Period', ' ': 'Space' };
  let key = namedKeys[event.key] || event.key;
  if (key.length === 1) key = key.toUpperCase();
  parts.push(key);
  return parts.join('+');
}

function handleKeyDown(event) {
  if (captureBinding) {
    event.preventDefault();
    const callback = captureBinding.done;
    const action = captureBinding.action;
    captureBinding = null;
    callback(event.key === 'Escape' ? null : comboFromEvent(event));
    return;
  }
  if (modalClose) {
    if (event.key === 'Escape') { event.preventDefault(); modalClose(); }
    return;
  }
  if (event.key === 'Escape') {
    closeMenus();
    clearSelectionState();
    render();
    event.preventDefault();
    return;
  }

  const combo = comboFromEvent(event);
  const boundAction = Object.keys(settings.keybindings).find((name) => settings.keybindings[name] === combo);
  if (boundAction) {
    event.preventDefault();
    handleAction(boundAction, event);
    return;
  }

  if (event.key === 'F3') { event.preventDefault(); findNext(event.shiftKey); return; }
  if (currentDoc()?.type === 'settings') return;

  const shift = event.shiftKey;
  let handled = true;
  switch (event.key) {
    case 'Backspace': handleBackspace(); break;
    case 'Delete': handleDelete(); break;
    case 'Enter': handleEnter(); break;
    case 'Tab': handleTab(); break;
    case 'ArrowLeft': moveCursor(-1, 0, shift); break;
    case 'ArrowRight': moveCursor(1, 0, shift); break;
    case 'ArrowUp': moveCursor(0, -1, shift); break;
    case 'ArrowDown': moveCursor(0, 1, shift); break;
    case 'Home': moveToLineStart(shift); break;
    case 'End': moveToLineEnd(shift); break;
    case 'PageUp': moveCursor(0, -10, shift); break;
    case 'PageDown': moveCursor(0, 10, shift); break;
    default:
      if (event.key.length === 1 && !event.altKey) insertCharacter(event.key);
      else handled = false;
  }
  if (handled) event.preventDefault();
}

function measureCharWidth() {
  if (charWidthCache !== null) return charWidthCache;
  const probe = document.createElement('span');
  probe.textContent = 'M'.repeat(20);
  probe.style.cssText = `position:absolute;visibility:hidden;white-space:pre;font-family:${settings.fontFamily};font-size:${settings.fontSize}px;`;
  document.body.appendChild(probe);
  charWidthCache = probe.getBoundingClientRect().width / 20 || 8;
  probe.remove();
  return charWidthCache;
}

function pointToPosition(event) {
  const element = document.elementFromPoint(event.clientX, event.clientY);
  const row = element?.closest('.line');
  if (!row) return null;
  const y = Number(row.dataset.line);
  if (!Number.isInteger(y) || state.buffer[y] === undefined) return null;
  const content = $('.line-content', row);
  const rect = content.getBoundingClientRect();
  const x = clamp(Math.round((event.clientX - rect.left) / measureCharWidth()), 0, state.buffer[y].length);
  return { x, y };
}

function setupMouse() {
  let dragging = false;
  els.editor.addEventListener('mousedown', (event) => {
    if (event.button !== 0 || modalClose) return;
    const pos = pointToPosition(event);
    if (!pos) return;
    state.cursorY = pos.y;
    state.cursorX = pos.x;
    state.selectionStartX = pos.x;
    state.selectionStartY = pos.y;
    state.selectionEndX = null;
    state.selectionEndY = null;
    dragging = true;
    render();
    event.preventDefault();
  });
  document.addEventListener('mousemove', (event) => {
    if (!dragging) return;
    const pos = pointToPosition(event);
    if (!pos) return;
    state.selectionEndX = pos.x;
    state.selectionEndY = pos.y;
    render();
  });
  document.addEventListener('mouseup', () => { dragging = false; });
  els.editor.addEventListener('contextmenu', (event) => {
    event.preventDefault();
    showContextMenu(event.clientX, event.clientY, [
      { label: 'Cut', icon: 'scissors', action: cut },
      { label: 'Copy', icon: 'copy', action: copy },
      { label: 'Paste', icon: 'clipboard', action: paste },
      { separator: true },
      { label: 'Select All', icon: 'check', action: selectAll },
    ]);
  });
}

function setupMenus() {
  $$('.menu-trigger').forEach((trigger) => {
    trigger.addEventListener('click', (event) => {
      event.stopPropagation();
      const menu = trigger.parentElement;
      const wasOpen = menu.classList.contains('open');
      closeMenus();
      if (!wasOpen) menu.classList.add('open');
    });
  });

  document.addEventListener('click', (event) => {
    closeMenus();
    const actionEl = event.target.closest('[data-action]');
    if (actionEl) { handleAction(actionEl.dataset.action, event); return; }
    const layoutEl = event.target.closest('[data-layout]');
    if (layoutEl) {
      const target = layoutEl.dataset.layout;
      if (target === 'side-left' || target === 'side-right') {
        layout.sideSide = target === 'side-right' ? 'right' : 'left';
        layout.sideVisible = true;
      } else {
        layout.panelPosition = target === 'panel-bottom' ? 'bottom' : target === 'panel-right' ? 'right' : 'left';
        layout.panelVisible = true;
      }
      applyLayout();
      return;
    }
    const closeEl = event.target.closest('[data-close]');
    if (closeEl) { event.stopPropagation(); closeTab(Number(closeEl.dataset.close)); return; }
    const tabEl = event.target.closest('.tab');
    if (tabEl) { switchTab(Number(tabEl.dataset.id)); return; }
    const viewEl = event.target.closest('[data-view]');
    if (viewEl) { layout.sideView = viewEl.dataset.view; layout.sideVisible = true; applyLayout(); renderSide(); return; }
    const panelTabEl = event.target.closest('.panel-tab');
    if (panelTabEl) { layout.panelView = panelTabEl.dataset.panel; renderPanel(); return; }
    const resultEl = event.target.closest('[data-result]');
    if (resultEl) { state.currentFindIndex = Number(resultEl.dataset.result); goToFindResult(state.currentFindIndex); }
  });
}

function switchTheme() {
  const themes = allThemes();
  const index = themes.findIndex((theme) => theme.id === settings.themeId);
  settings.themeId = themes[(index + 1) % themes.length].id;
  charWidthCache = null;
  saveSettings();
  render();
  showSnackbar(`Switched to ${activeTheme().name}`);
}

function zoomIn() { settings.fontSize = clamp(settings.fontSize + 2, 8, 32); charWidthCache = null; saveSettings(); render(); }
function zoomOut() { settings.fontSize = clamp(settings.fontSize - 2, 8, 32); charWidthCache = null; saveSettings(); render(); }

/* ---------------- Folders & file tree ---------------- */

async function pickFolder() {
  const result = await invoke('plugin:dialog|open', {
    options: { directory: true, multiple: false, title: 'Open Folder' },
  });
  return typeof result === 'string' ? result : null;
}

async function openFolder() {
  try {
    const path = await pickFolder();
    if (!path) return;
    workspaceFolders = [path];
    directoryCache.clear();
    expandedDirs.add(path);
    saveWorkspace();
    renderAll();
    showSnackbar(`Opened folder: ${baseName(path)}`, '#16825d');
  } catch (error) {
    showErrorDialog('Open Folder Error', error.message || String(error));
  }
}

async function addFolder() {
  try {
    const path = await pickFolder();
    if (!path || workspaceFolders.includes(path)) return;
    workspaceFolders.push(path);
    expandedDirs.add(path);
    saveWorkspace();
    renderAll();
    showSnackbar(`Added folder: ${baseName(path)}`, '#16825d');
  } catch (error) {
    showErrorDialog('Open Folder Error', error.message || String(error));
  }
}

function removeFolder(path) {
  workspaceFolders = workspaceFolders.filter((folder) => folder !== path);
  saveWorkspace();
  renderSide();
}

function addRecent(path) {
  recentFiles = [path, ...recentFiles.filter((item) => item !== path)].slice(0, settings.recentFilesLimit);
  saveWorkspace();
}

async function openPath(path) {
  try {
    const existing = docs.find((doc) => doc.path === path);
    if (existing) { switchTab(existing.id); return; }
    const content = await readFileText(path);
    const proceed = () => { makeDoc(baseName(path), path, content, true); addRecent(path); };
    if (settings.confirmUnsaved && state.isModified) showUnsavedDialog(proceed);
    else proceed();
  } catch (error) {
    showErrorDialog('Open File Error', error.message || String(error));
  }
}

async function getDirectoryEntries(path) {
  if (directoryCache.has(path)) return directoryCache.get(path);
  const entries = await invoke('list_directory', { path });
  directoryCache.set(path, entries);
  return entries;
}

function renderTreeNode(container, path, depth, name, isDirectory) {
  const row = document.createElement('button');
  row.className = 'tree-row';
  row.style.paddingLeft = `${8 + depth * 13}px`;
  const open = docs.some((doc) => doc.path === path);
  const dirty = docs.some((doc) => doc.path === path && doc.snapshot?.isModified);
  const caret = isDirectory ? (expandedDirs.has(path) ? 'chevron-down' : 'chevron-right') : '';
  row.innerHTML = `
    <span class="tree-caret">${caret ? iconSvg(caret, 13) : ''}</span>
    <span class="tree-icon">${iconSvg(isDirectory ? 'folder' : 'file', 14)}</span>
    <span class="row-name">${esc(name)}</span>
    ${dirty ? '<span class="dirty">&#8226;</span>' : ''}`;
  if (open) row.classList.add('open-file');
  row.addEventListener('click', async () => {
    if (!isDirectory) { openPath(path); return; }
    if (expandedDirs.has(path)) expandedDirs.delete(path);
    else {
      expandedDirs.add(path);
      if (!directoryCache.has(path)) {
        try { await getDirectoryEntries(path); }
        catch (error) { showErrorDialog('Folder Error', error.message || String(error)); }
      }
    }
    renderSide();
  });
  container.appendChild(row);
  if (isDirectory && expandedDirs.has(path) && directoryCache.has(path)) {
    directoryCache.get(path).forEach((entry) => {
      renderTreeNode(container, entry.path, depth + 1, entry.name, entry.is_dir);
    });
  }
}

function renderFoldersSection() {
  if (!workspaceFolders.length) return null;
  return section('FOLDERS', 'chevron-down', () => {
    const list = document.createElement('div');
    list.className = 'side-list tree';
    workspaceFolders.forEach((folder) => renderTreeNode(list, folder, 0, baseName(folder), true));
    return list;
  });
}

function renderRecentSection() {
  if (!recentFiles.length) return null;
  return section('RECENT', 'chevron-down', () => {
    const list = document.createElement('div');
    list.className = 'side-list';
    recentFiles.forEach((path) => {
      const row = document.createElement('button');
      row.className = 'side-row';
      row.innerHTML = `<span class="row-icon">${iconSvg('clock', 14)}</span><span class="row-name">${esc(baseName(path))}</span>`;
      row.title = path;
      row.addEventListener('click', () => openPath(path));
      list.appendChild(row);
    });
    return list;
  });
}

/* ---------------- Settings ---------------- */

const SETTINGS_CATEGORIES = [
  { id: 'general', name: 'General', icon: 'settings' },
  { id: 'editor', name: 'Editor', icon: 'edit-3' },
  { id: 'appearance', name: 'Appearance', icon: 'droplet' },
  { id: 'themes', name: 'Themes', icon: 'layers' },
  { id: 'workspace', name: 'Files & Workspace', icon: 'folder' },
  { id: 'keybindings', name: 'Keybindings', icon: 'layout' },
];

const FONT_OPTIONS = [
  ["Consolas, 'SF Mono', 'Courier New', monospace", 'Consolas / SF Mono'],
  ["'Courier New', monospace", 'Courier New'],
  ["'JetBrains Mono', 'Fira Code', Consolas, monospace", 'JetBrains Mono / Fira Code'],
  ["'DejaVu Sans Mono', monospace", 'DejaVu Sans Mono'],
  ["'Liberation Mono', monospace", 'Liberation Mono'],
];

const KEYBINDING_LABELS = {
  newFile: 'New File', openFile: 'Open File', openFolder: 'Open Folder', save: 'Save',
  saveAs: 'Save As', exit: 'Exit', undo: 'Undo', redo: 'Redo', cut: 'Cut',
  copy: 'Copy', copyAll: 'Copy All', paste: 'Paste', selectAll: 'Select All',
  find: 'Find', replace: 'Replace', theme: 'Switch Theme', zoomIn: 'Zoom In',
  zoomOut: 'Zoom Out', toggleSide: 'Toggle Side Bar', togglePanel: 'Toggle Panel',
  settings: 'Open Settings',
};

let settingsCategory = 'general';

function openSettings() {
  const existing = docs.find((doc) => doc.type === 'settings');
  if (existing) { switchTab(existing.id); return; }
  const doc = {
    id: ++docId, name: 'Settings', path: null, type: 'settings',
    snapshot: { currentFileName: 'Settings', isModified: false, undoStack: [], redoStack: [] },
  };
  docs.push(doc);
  activeId = doc.id;
  renderAll();
}

function settingRow(label, description, control) {
  return `<div class="setting-row"><div class="setting-info"><div class="setting-label">${label}</div><div class="setting-desc">${description}</div></div><div class="setting-control">${control}</div></div>`;
}

function checkSetting(key, label, description) {
  return settingRow(label, description, `<input type="checkbox" data-setting="${key}" ${settings[key] ? 'checked' : ''} />`);
}

function selectSetting(key, label, description, options) {
  return settingRow(label, description, `<select data-setting="${key}">${options.map(([value, text]) => `<option value="${esc(value)}" ${settings[key] === value ? 'selected' : ''}>${esc(text)}</option>`).join('')}</select>`);
}

function rangeSetting(key, label, description, min, max, step = 1) {
  return settingRow(label, description, `<div class="range-wrap"><input type="range" data-setting="${key}" min="${min}" max="${max}" step="${step}" value="${settings[key]}" /><span data-value-for="${key}">${settings[key]}</span></div>`);
}

function layoutSelect(key, label, description, options) {
  return settingRow(label, description, `<select data-layout-setting="${key}">${options.map(([value, text]) => `<option value="${value}" ${layout[key] === value ? 'selected' : ''}>${text}</option>`).join('')}</select>`);
}

function layoutCheck(key, label, description) {
  return settingRow(label, description, `<input type="checkbox" data-layout-setting="${key}" ${layout[key] ? 'checked' : ''} />`);
}

function renderSettingsCategory(category) {
  if (category === 'general') {
    return `
      <h2>General</h2>
      <p class="settings-hint">Application-wide behavior and chrome.</p>
      ${selectSetting('uiDensity', 'UI Density', 'Controls the spacing of menus, panels and lists.', [['compact', 'Compact'], ['default', 'Default'], ['spacious', 'Spacious']])}
      ${checkSetting('showMenuBar', 'Show Menu Bar', 'Display the File / Edit / View / Help menu bar.')}
      ${checkSetting('showStatusBar', 'Show Status Bar', 'Display the status bar at the bottom of the window.')}
      ${checkSetting('showTitleFile', 'Show File in Title', 'Display the active file name in the title bar.')}
      ${checkSetting('confirmUnsaved', 'Confirm Unsaved Changes', 'Ask before discarding unsaved edits.')}
      ${checkSetting('restoreWorkspace', 'Restore Workspace Folders', 'Reopen the last folder(s) on launch.')}
      ${checkSetting('overrideAccent', 'Override Accent Color', 'Use a custom accent instead of the theme accent.')}
      ${settingRow('Accent Color', 'Used for selection highlights, focus and status bar.', `<input type="color" data-setting="accentColor" value="${settings.accentColor}" />`)}`;
  }
  if (category === 'editor') {
    return `
      <h2>Editor</h2>
      <p class="settings-hint">Fine-tune how text looks and behaves.</p>
      ${rangeSetting('fontSize', 'Font Size', 'Editor font size in pixels.', 8, 32)}
      ${selectSetting('fontFamily', 'Font Family', 'The font used in the editor.', FONT_OPTIONS)}
      ${rangeSetting('tabSize', 'Tab Size', 'Number of spaces inserted for Tab.', 2, 8)}
      ${checkSetting('insertSpaces', 'Insert Spaces', 'Use spaces instead of tab characters.')}
      ${rangeSetting('lineHeight', 'Line Height', 'Height of each editor line in pixels.', 16, 40)}
      ${checkSetting('showLineNumbers', 'Show Line Numbers', 'Display line numbers in the gutter.')}
      ${checkSetting('highlightActiveLine', 'Highlight Active Line', 'Shade the line containing the cursor.')}
      ${checkSetting('wordWrap', 'Word Wrap', 'Wrap long lines instead of scrolling horizontally.')}
      ${checkSetting('showWhitespace', 'Render Whitespace', 'Show spaces and tabs as visible marks.')}
      ${selectSetting('cursorStyle', 'Cursor Style', 'Shape of the text caret.', [['line', 'Line'], ['block', 'Block'], ['underline', 'Underline']])}
      ${rangeSetting('cursorBlink', 'Cursor Blink', 'Blink interval in milliseconds. 0 disables blinking.', 0, 1000, 50)}`;
  }
  if (category === 'appearance') {
    return `
      <h2>Appearance</h2>
      <p class="settings-hint">Layout and which parts of the workspace are visible.</p>
      ${selectSetting('themeId', 'Color Theme', 'The active editor and UI theme.', allThemes().map((theme) => [theme.id, theme.name]))}
      ${layoutCheck('activityBarVisible', 'Show Activity Bar', 'The vertical icon strip on the side.')}
      ${layoutCheck('sideVisible', 'Show Side Bar', 'Explorer, Search and Source Control.')}
      ${layoutCheck('panelVisible', 'Show Panel', 'Problems, Output and Search results.')}
      ${layoutSelect('activitySide', 'Activity Bar Position', 'Dock the activity bar on either side.', [['left', 'Left'], ['right', 'Right']])}
      ${layoutSelect('sideSide', 'Side Bar Position', 'Dock the side bar on either side.', [['left', 'Left'], ['right', 'Right']])}
      ${layoutSelect('panelPosition', 'Panel Position', 'Dock the panel to an edge.', [['bottom', 'Bottom'], ['right', 'Right'], ['left', 'Left']])}
      ${rangeSetting('sideWidth', 'Side Bar Width', 'Width of the side bar in pixels.', 160, 560)}
      ${rangeSetting('panelSize', 'Panel Size', 'Height or width of the panel in pixels.', 90, 640)}
      ${settingRow('Reset Layout', 'Restore the default panel arrangement.', '<button class="btn" data-settings-action="resetLayout">Reset Layout</button>')}`;
  }
  if (category === 'themes') {
    const cards = allThemes().map((theme) => {
      const active = theme.id === settings.themeId;
      const custom = theme.id.startsWith('custom-');
      return `<div class="theme-card ${active ? 'selected' : ''}">
        <div class="theme-swatch">
          <span style="background:${theme.chrome}"></span><span style="background:${theme.bg}"></span>
          <span style="background:${theme.text}"></span><span style="background:${theme.accent}"></span>
        </div>
        <div class="theme-card-name">${esc(theme.name)}</div>
        <div class="theme-card-meta">${custom ? 'Custom' : 'Built-in'}</div>
        <div class="theme-actions">
          <button class="btn" data-theme-use="${theme.id}">${active ? 'Active' : 'Use'}</button>
          ${custom ? `<button class="btn" data-theme-edit="${theme.id}">Edit</button><button class="btn" data-theme-delete="${theme.id}">Delete</button>` : ''}
        </div>
      </div>`;
    }).join('');
    return `
      <h2>Themes</h2>
      <p class="settings-hint">Choose a built-in theme or design your own. Custom themes are saved on this device.</p>
      <div class="settings-buttons">
        <button class="btn btn-primary" data-settings-action="createTheme">Create Custom Theme</button>
        <button class="btn" data-settings-action="exportThemes">Export Themes</button>
        <button class="btn" data-settings-action="importThemes">Import Themes</button>
      </div>
      <div class="theme-grid">${cards}</div>`;
  }
  if (category === 'workspace') {
    const folders = workspaceFolders.map((folder) => `
      <div class="folder-row"><span>${iconSvg('folder', 14)}</span><span class="row-name" title="${esc(folder)}">${esc(folder)}</span><button class="btn" data-remove-folder="${esc(folder)}">Remove</button></div>`).join('') || '<p class="settings-hint">No folder is open.</p>';
    return `
      <h2>Files &amp; Workspace</h2>
      <p class="settings-hint">Open a directory to browse and edit its files, or add several folders.</p>
      <div class="settings-buttons">
        <button class="btn btn-primary" data-settings-action="openFolder">Open Folder</button>
        <button class="btn" data-settings-action="addFolder">Add Folder</button>
      </div>
      <div class="folder-list">${folders}</div>
      ${selectSetting('autoSave', 'Auto Save', 'Automatically save edits.', [['off', 'Off'], ['delay', 'After a delay'], ['focus', 'When the window loses focus']])}
      ${rangeSetting('autoSaveDelay', 'Auto Save Delay', 'Milliseconds of inactivity before saving.', 300, 5000, 100)}
      ${rangeSetting('recentFilesLimit', 'Recent Files', 'How many recently opened files to remember.', 0, 30)}`;
  }
  const bindings = Object.keys(KEYBINDING_LABELS).map((action) => `
    <div class="keybind-row">
      <span>${KEYBINDING_LABELS[action]}</span>
      <code>${esc(settings.keybindings[action] || 'Unassigned')}</code>
      <button class="btn" data-keybind="${action}">Change...</button>
    </div>`).join('');
  return `
    <h2>Keybindings</h2>
    <p class="settings-hint">Click Change, then press the key combination you want. Press Escape to cancel.</p>
    <div class="settings-buttons"><button class="btn" data-settings-action="resetKeys">Restore Defaults</button></div>
    <div class="keybind-list">${bindings}</div>`;
}

function renderSettingsView() {
  const editor = els.editor;
  editor.classList.remove('wrap', 'no-highlight');
  const nav = SETTINGS_CATEGORIES.map((category) => `
    <button class="settings-nav ${settingsCategory === category.id ? 'active' : ''}" data-category="${category.id}">
      ${iconSvg(category.icon, 15)}<span>${category.name}</span>
    </button>`).join('');
  const view = document.createElement('div');
  view.className = 'settings-view';
  view.innerHTML = `
    <aside class="settings-side"><div class="settings-title">${iconSvg('settings', 18)}<span>Settings</span></div>${nav}</aside>
    <div class="settings-main"><div class="settings-content">${renderSettingsCategory(settingsCategory)}</div></div>`;
  editor.replaceChildren(view);
  hydrateIcons(view);
  $$('.settings-nav', view).forEach((button) => {
    button.addEventListener('click', () => { settingsCategory = button.dataset.category; render(); });
  });
  bindSettingsControls(view);
}

function bindSettingsControls(root) {
  $$('[data-setting]', root).forEach((element) => {
    const key = element.dataset.setting;
    const apply = () => {
      let value = element.value;
      if (element.type === 'checkbox') value = element.checked;
      else if (element.type === 'range' || element.type === 'number') value = Number(value);
      settings[key] = value;
      const display = $(`[data-value-for="${key}"]`, root);
      if (display) display.textContent = String(value);
      saveSettings();
      charWidthCache = null;
      applyTheme();
      applyLayout();
      render();
    };
    element.addEventListener('change', apply);
  });

  $$('[data-layout-setting]', root).forEach((element) => {
    const key = element.dataset.layoutSetting;
    const apply = () => {
      layout[key] = element.type === 'checkbox' ? element.checked
        : (element.type === 'range' || element.type === 'number') ? Number(element.value) : element.value;
      const display = $(`[data-value-for="${key}"]`, root);
      if (display) display.textContent = String(layout[key]);
      saveLayout();
      applyLayout();
      render();
    };
    element.addEventListener('change', apply);
  });

  $$('[data-theme-use]', root).forEach((button) => button.addEventListener('click', () => {
    settings.themeId = button.dataset.themeUse;
    saveSettings();
    charWidthCache = null;
    render();
    showSnackbar(`Theme: ${activeTheme().name}`);
  }));
  $$('[data-theme-edit]', root).forEach((button) => button.addEventListener('click', () => {
    themeEditorModal(customThemes.find((theme) => theme.id === button.dataset.themeEdit));
  }));
  $$('[data-theme-delete]', root).forEach((button) => button.addEventListener('click', () => {
    const theme = customThemes.find((item) => item.id === button.dataset.themeDelete);
    if (!theme) return;
    customThemes = customThemes.filter((item) => item.id !== theme.id);
    if (settings.themeId === theme.id) settings.themeId = 'light';
    saveCustomThemes();
    saveSettings();
    render();
    showSnackbar(`Deleted theme: ${theme.name}`);
  }));
  $$('[data-remove-folder]', root).forEach((button) => button.addEventListener('click', () => {
    removeFolder(button.dataset.removeFolder);
    render();
  }));
  $$('[data-keybind]', root).forEach((button) => button.addEventListener('click', () => {
    const previous = button.textContent;
    button.textContent = 'Press keys...';
    captureBinding = {
      action: button.dataset.keybind,
      done: (combo) => {
        if (combo) {
          Object.keys(settings.keybindings).forEach((action) => {
            if (settings.keybindings[action] === combo) settings.keybindings[action] = '';
          });
          settings.keybindings[button.dataset.keybind] = combo;
          saveSettings();
        }
        if (button.isConnected) button.textContent = previous;
        render();
      },
    };
  }));

  $$('[data-settings-action]', root).forEach((button) => button.addEventListener('click', () => {
    const action = button.dataset.settingsAction;
    if (action === 'resetLayout') { resetLayout(); render(); }
    if (action === 'openFolder') openFolder();
    if (action === 'addFolder') addFolder();
    if (action === 'createTheme') themeEditorModal(null);
    if (action === 'exportThemes') exportThemes();
    if (action === 'importThemes') importThemes();
    if (action === 'resetKeys') {
      settings.keybindings = { ...DEFAULT_KEYBINDINGS };
      saveSettings();
      render();
      showSnackbar('Keybindings restored');
    }
  }));
}

async function exportThemes() {
  try {
    await clipboardWrite(JSON.stringify(customThemes, null, 2));
    showSnackbar('Custom themes copied to clipboard', '#16825d');
  } catch (error) { showErrorDialog('Export Error', error.message); }
}

function importThemes() {
  openModal({
    title: 'Import Themes',
    icon: iconSvg('download'),
    bodyHtml: '<div class="modal-field"><label>Paste exported theme JSON</label><textarea id="themeJson" rows="8" style="width:100%;padding:8px;border:1px solid var(--border-strong);border-radius:4px;background:var(--input);color:var(--chrome-text);font-family:inherit"></textarea></div>',
    actions: [
      { label: 'Cancel', onClick: (close) => close() },
      {
        label: 'Import', primary: true,
        onClick: (close, root) => {
          try {
            const incoming = JSON.parse($('#themeJson', root).value);
            if (!Array.isArray(incoming)) throw new Error('Expected an array of themes.');
            incoming.forEach((theme) => {
              if (!theme.id || !theme.name) throw new Error('Each theme needs an id and a name.');
              COLOR_KEYS.forEach((key) => { if (!theme[key]) theme[key] = activeTheme()[key]; });
              const index = customThemes.findIndex((item) => item.id === theme.id);
              if (index >= 0) customThemes[index] = theme; else customThemes.push(theme);
            });
            saveCustomThemes();
            close();
            render();
            showSnackbar(`Imported ${incoming.length} theme(s)`, '#16825d');
          } catch (error) { showErrorDialog('Import Error', error.message); }
        },
      },
    ],
  });
}

function themeEditorModal(theme) {
  const isNew = !theme;
  const base = activeTheme();
  const draft = theme ? { ...theme } : {
    id: `custom-${Date.now()}`, name: 'My Theme',
    ...Object.fromEntries(COLOR_KEYS.map((key) => [key, base[key]])),
  };
  const colors = COLOR_KEYS.map((key) => `
    <label class="theme-color-field"><span>${key}</span><input type="color" data-color="${key}" value="${draft[key]}" /></label>`).join('');
  openModal({
    title: isNew ? 'Create Custom Theme' : `Edit ${theme.name}`,
    icon: iconSvg('droplet'),
    bodyHtml: `
      <div class="modal-field"><label>Theme name</label><input id="themeName" value="${esc(draft.name)}" /></div>
      <div class="theme-colors">${colors}</div>`,
    actions: [
      { label: 'Cancel', onClick: (close) => close() },
      {
        label: isNew ? 'Create' : 'Save', primary: true,
        onClick: (close, root) => {
          draft.name = $('#themeName', root).value.trim() || 'Custom Theme';
          $$('[data-color]', root).forEach((input) => { draft[input.dataset.color] = input.value; });
          if (isNew) customThemes.push(draft);
          else Object.assign(theme, draft);
          saveCustomThemes();
          settings.themeId = draft.id;
          saveSettings();
          charWidthCache = null;
          close();
          render();
          showSnackbar(`Theme saved: ${draft.name}`, '#16825d');
        },
      },
    ],
  });
}

function init() {
  Object.assign(els, {
    workspace: $('#workspace'),
    activityBar: $('#activityBar'),
    sideBar: $('#sideBar'),
    sideTitle: $('#sideTitle'),
    sideBody: $('#sideBody'),
    sideSplitter: $('#sideSplitter'),
    center: $('#center'),
    editorZone: $('#editorZone'),
    tabs: $('#tabs'),
    editor: $('#editor'),
    panelSplitter: $('#panelSplitter'),
    panel: $('#panel'),
    panelHeader: $('#panelHeader'),
    panelBody: $('#panelBody'),
    contextMenu: $('#contextMenu'),
    modalRoot: $('#modalRoot'),
    snackbar: $('#snackbar'),
  });

  loadSettings();
  loadCustomThemes();
  loadWorkspace();
  if (!settings.restoreWorkspace) workspaceFolders = [];
  loadLayout();
  hydrateIcons();
  setupMenus();
  setupMouse();
  setupSplitters();
  setupDragging();
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('blur', () => {
    if (settings.autoSave === 'focus' && state.isModified) saveFile();
  });

  let lastBlink = Date.now();
  setInterval(() => {
    const now = Date.now();
    if (settings.cursorBlink > 0 && now - lastBlink >= settings.cursorBlink) {
      $('#cursor')?.classList.toggle('off');
      lastBlink = now;
    }
  }, 50);

  makeDoc('Untitled-1', null);
  logOutput('Pezli Editor ready');
}

init();
