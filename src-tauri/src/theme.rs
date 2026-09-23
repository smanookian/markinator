// Read Omarchy theme colors and the system monospace font.
// Watch the theme dir and tell the frontend when colors change.

use notify::{RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Emitter};

const KEYS: &[&str] = &[
    "mode",
    "accent",
    "selection",
    "muted",
    "background",
    "dark_background",
    "darker_background",
    "lighter_background",
    "foreground",
    "dark_foreground",
    "light_foreground",
    "bright_foreground",
    "red",
    "yellow",
    "orange",
    "green",
    "cyan",
    "blue",
    "magenta",
];

const FALLBACK: &[(&str, &str)] = &[
    ("mode", "dark"),
    ("accent", "#89b4fa"),
    ("selection", "#45475a"),
    ("muted", "#585b70"),
    ("background", "#1e1e2e"),
    ("dark_background", "#161622"),
    ("darker_background", "#101019"),
    ("lighter_background", "#313244"),
    ("foreground", "#cdd6f4"),
    ("dark_foreground", "#6c7086"),
    ("light_foreground", "#bac2de"),
    ("bright_foreground", "#cdd6f4"),
    ("red", "#f38ba8"),
    ("yellow", "#f9e2af"),
    ("orange", "#f6b6ab"),
    ("green", "#a6e3a1"),
    ("cyan", "#94e2d5"),
    ("blue", "#89b4fa"),
    ("magenta", "#f5c2e7"),
];

#[derive(Serialize, Clone)]
pub struct Theme {
    pub colors: BTreeMap<String, String>,
    pub font: String,
}

fn omarchy_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state/omarchy/current"))
}

fn read_colors() -> BTreeMap<String, String> {
    let mut colors: BTreeMap<String, String> = FALLBACK
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let Some(dir) = omarchy_dir() else { return colors };
    let Ok(text) = std::fs::read_to_string(dir.join("theme/colors.toml")) else {
        return colors;
    };
    let Ok(table) = text.parse::<toml::Table>() else { return colors };
    for key in KEYS {
        if let Some(v) = table.get(*key).and_then(|v| v.as_str()) {
            colors.insert(key.to_string(), v.to_string());
        }
    }
    colors
}

fn read_font() -> String {
    let out = Command::new("fc-match")
        .args(["monospace", "-f", "%{family}"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let first = out.split(',').next().unwrap_or("").trim();
    if first.is_empty() {
        "monospace".to_string()
    } else {
        first.to_string()
    }
}

pub fn current() -> Theme {
    Theme {
        colors: read_colors(),
        font: read_font(),
    }
}

// Watch the Omarchy "current" dir (the theme symlink lives there) and the
// theme dir itself. Any change means: re-read colors and emit "theme".
// The caller keeps the returned watcher alive.
pub fn watch(app: AppHandle) -> Option<notify::RecommendedWatcher> {
    let dir = omarchy_dir()?;
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            let _ = app.emit("theme", current());
        }
    })
    .ok()?;
    let _ = watcher.watch(&dir, RecursiveMode::NonRecursive);
    if let Ok(theme_dir) = std::fs::canonicalize(dir.join("theme")) {
        let _ = watcher.watch(&theme_dir, RecursiveMode::NonRecursive);
    }
    Some(watcher)
}
