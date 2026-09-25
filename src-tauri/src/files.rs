// Open, save, render, and watch one markdown file.

use notify::{RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, State, Window};

#[derive(Serialize)]
pub struct Loaded {
    pub path: String,
    pub text: String,
    pub exists: bool,
}

// Make a path absolute. Keeps the path even if the file does not exist yet.
fn absolute(path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map(|d| d.join(p)).unwrap_or_else(|_| p.to_path_buf())
    }
}

#[tauri::command]
pub fn load_file(path: String) -> Result<Loaded, String> {
    let abs = absolute(&path);
    let path = abs.to_string_lossy().into_owned();
    if !abs.exists() {
        return Ok(Loaded { path, text: String::new(), exists: false });
    }
    let bytes = std::fs::read(&abs).map_err(|e| format!("Cannot read {path}: {e}"))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{path} is not UTF-8 text"))?;
    Ok(Loaded { path, text, exists: true })
}

#[tauri::command]
pub fn save_file(path: String, text: String) -> Result<String, String> {
    let abs = absolute(&path);
    std::fs::write(&abs, text).map_err(|e| format!("Cannot save {}: {e}", abs.display()))?;
    Ok(abs.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn render(text: String) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let mut out = String::with_capacity(text.len() * 2);
    html::push_html(&mut out, Parser::new_ext(&text, opts));
    out
}

// Watches the open file of each window, by window label.
// Emits "file-changed" to that window when its file changes on disk.
pub struct FileWatch {
    watchers: Mutex<HashMap<String, notify::RecommendedWatcher>>,
}

impl FileWatch {
    pub fn new() -> Self {
        Self { watchers: Mutex::new(HashMap::new()) }
    }

    // Stop watching for a closed window.
    pub fn forget(&self, label: &str) {
        self.watchers.lock().remove(label);
    }
}

#[tauri::command]
pub fn watch_file(app: AppHandle, window: Window, state: State<FileWatch>, path: Option<String>) {
    let label = window.label().to_string();
    // Dropping the old watcher stops it.
    state.forget(&label);
    let Some(path) = path else { return };
    let abs = absolute(&path);
    let target = label.clone();
    let Ok(mut w) = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res {
            if ev.kind.is_modify() || ev.kind.is_create() {
                let _ = app.emit_to(&target, "file-changed", ());
            }
        }
    }) else {
        return;
    };
    if w.watch(&abs, RecursiveMode::NonRecursive).is_ok() {
        state.watchers.lock().insert(label, w);
    }
}
