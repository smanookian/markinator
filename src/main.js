// markinator frontend. One textarea, one view, a few keys.

const { invoke, convertFileSrc } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const dialog = window.__TAURI__.dialog;
const opener = window.__TAURI__.opener;
const win = window.__TAURI__.window.getCurrentWindow();
const webview = window.__TAURI__.webview.getCurrentWebview();

const editor = document.getElementById("editor");
const view = document.getElementById("view");
const status = document.getElementById("status");
const statusMode = document.getElementById("status-mode");
const statusFile = document.getElementById("status-file");
const statusMsg = document.getElementById("status-msg");

const FILTERS = [{ name: "Markdown", extensions: ["md", "markdown", "txt"] }];

let path = null; // absolute path of the open file, or null when untitled
let dirty = false;
let mode = "edit";
let zoom = 1;

// --- title and status ---

function fileName() {
  return path ? path.slice(path.lastIndexOf("/") + 1) : "Untitled";
}

function dirName() {
  return path ? path.slice(0, path.lastIndexOf("/")) : null;
}

function updateTitle() {
  win.setTitle(`${dirty ? "*" : ""}${fileName()} - markinator`);
  statusFile.textContent = fileName();
}

function setDirty(v) {
  if (dirty === v) return;
  dirty = v;
  updateTitle();
}

let statusTimer = null;
function showStatus() {
  status.hidden = false;
  clearTimeout(statusTimer);
  statusTimer = setTimeout(() => { status.hidden = true; }, 2000);
}

function message(text) {
  statusMsg.textContent = text;
  showStatus();
}

// --- theme ---

function applyTheme(t) {
  const root = document.documentElement.style;
  for (const [k, v] of Object.entries(t.colors)) {
    root.setProperty("--" + k.replace(/_/g, "-"), v);
  }
  root.setProperty("--font", `"${t.font}"`);
}

// --- modes ---

function scrollRatio(el) {
  const max = el.scrollHeight - el.clientHeight;
  return max > 0 ? el.scrollTop / max : 0;
}

function setScrollRatio(el, r) {
  el.scrollTop = r * (el.scrollHeight - el.clientHeight);
}

function fixLinks() {
  const dir = dirName();
  for (const img of view.querySelectorAll("img[src]")) {
    const src = img.getAttribute("src");
    if (/^[a-z]+:/i.test(src)) continue;
    if (dir) img.src = convertFileSrc(src.startsWith("/") ? src : `${dir}/${src}`);
  }
  for (const code of view.querySelectorAll("pre code")) {
    hljs.highlightElement(code);
  }
}

async function renderView() {
  const text = editor.value;
  if (text.trim() === "") {
    view.textContent = "Empty. Press Ctrl+E to write.";
    view.classList.add("empty");
    return;
  }
  view.classList.remove("empty");
  view.innerHTML = await invoke("render", { text });
  fixLinks();
}

async function setMode(m) {
  const from = mode === "edit" ? editor : view;
  const ratio = scrollRatio(from);
  mode = m;
  statusMode.textContent = m.toUpperCase();
  if (m === "view") {
    await renderView();
    editor.hidden = true;
    view.hidden = false;
    setScrollRatio(view, ratio);
  } else {
    view.hidden = true;
    editor.hidden = false;
    setScrollRatio(editor, ratio);
    editor.focus();
  }
}

function toggleMode() {
  setMode(mode === "edit" ? "view" : "edit");
}

// --- files ---

async function loadPath(p, startMode) {
  let loaded;
  try {
    loaded = await invoke("load_file", { path: p });
  } catch (e) {
    await invoke("show_error", { text: String(e) });
    loaded = null;
  }
  path = loaded ? loaded.path : null;
  editor.value = loaded ? loaded.text : "";
  setDirty(false);
  updateTitle();
  invoke("watch_file", { path: loaded && loaded.exists ? path : null });
  if (loaded && !loaded.exists) {
    message("New file");
    await setMode("edit");
  } else {
    await setMode(startMode);
  }
}

async function reloadFromDisk() {
  if (!path || dirty) return;
  let loaded;
  try {
    loaded = await invoke("load_file", { path });
  } catch {
    return;
  }
  if (!loaded.exists || loaded.text === editor.value) return;
  const ratio = scrollRatio(mode === "edit" ? editor : view);
  editor.value = loaded.text;
  if (mode === "view") {
    await renderView();
    setScrollRatio(view, ratio);
  }
}

async function save(as) {
  let target = path;
  if (as || !target) {
    target = await dialog.save({
      filters: FILTERS,
      defaultPath: path || `${dirName() || ""}/Untitled.md`.replace(/^\//, ""),
    });
    if (!target) return false;
  }
  try {
    path = await invoke("save_file", { path: target, text: editor.value });
  } catch (e) {
    await invoke("show_error", { text: String(e) });
    return false;
  }
  setDirty(false);
  updateTitle();
  invoke("watch_file", { path });
  message("Saved");
  return true;
}

// Ask about unsaved changes. Returns true when it is ok to go on.
async function confirmDiscard() {
  if (!dirty) return true;
  const r = await invoke("ask_save");
  if (r === "save") return save(false);
  return r === "discard";
}

async function open() {
  if (!(await confirmDiscard())) return;
  const p = await dialog.open({ filters: FILTERS, multiple: false, directory: false });
  if (p) await loadPath(p, "view");
}

async function newDoc() {
  if (!(await confirmDiscard())) return;
  path = null;
  editor.value = "";
  setDirty(false);
  updateTitle();
  invoke("watch_file", { path: null });
  await setMode("edit");
}

async function quit() {
  if (await confirmDiscard()) win.destroy();
}

// --- links in view mode ---

view.addEventListener("click", async (e) => {
  const a = e.target.closest("a[href]");
  if (!a) return;
  const href = a.getAttribute("href");
  if (href.startsWith("#")) return;
  e.preventDefault();
  if (/^https?:/i.test(href)) {
    opener.openUrl(href);
    return;
  }
  if (/^[a-z]+:/i.test(href)) return;
  const clean = href.split("#")[0];
  if (!/\.(md|markdown)$/i.test(clean)) return;
  if (!(await confirmDiscard())) return;
  const dir = dirName();
  const target = clean.startsWith("/") || !dir ? clean : `${dir}/${clean}`;
  await loadPath(target, "view");
});

// --- keys ---

function setZoom(z) {
  zoom = Math.min(3, Math.max(0.5, z));
  webview.setZoom(zoom);
}

window.addEventListener("keydown", (e) => {
  showStatus();
  if (!e.ctrlKey) {
    if (e.key === "Tab" && e.target === editor) {
      e.preventDefault();
      editor.setRangeText("  ", editor.selectionStart, editor.selectionEnd, "end");
      setDirty(true);
    }
    return;
  }
  const key = e.key.toLowerCase();
  let handled = true;
  if (key === "e") toggleMode();
  else if (key === "o") open();
  else if (key === "s") save(e.shiftKey);
  else if (key === "n") newDoc();
  else if (key === "q") quit();
  else if (key === "+" || key === "=") setZoom(zoom + 0.1);
  else if (key === "-") setZoom(zoom - 0.1);
  else if (key === "0") setZoom(1);
  else handled = false;
  if (handled) e.preventDefault();
});

window.addEventListener("mousemove", showStatus);
editor.addEventListener("input", () => setDirty(true));
window.addEventListener("unhandledrejection", (e) => message(String(e.reason)));
window.addEventListener("error", (e) => message(e.message));

// --- start ---

let reloadTimer = null;
listen("file-changed", () => {
  clearTimeout(reloadTimer);
  reloadTimer = setTimeout(reloadFromDisk, 150);
});
listen("theme", (ev) => applyTheme(ev.payload));
listen("close-requested", quit);

(async () => {
  applyTheme(await invoke("theme"));
  const start = await invoke("start");
  if (start.file) await loadPath(start.file, start.edit ? "edit" : "view");
  else await setMode("edit");
  updateTitle();
  showStatus();
  win.show();
})();
