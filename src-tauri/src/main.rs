// markinator: write markdown, press Ctrl+E, read it.

mod files;
mod theme;

use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult};

const HELP: &str = "\
markinator - markdown editor and reader

Usage: markinator [options] [file]

Options:
  -e, --edit      start in edit mode (default is view mode when a file is given)
  -h, --help      show this help
  -v, --version   show the version (-V works too)

Keys: Ctrl+E toggle edit/view, Ctrl+O open, Ctrl+S save, Ctrl+Q quit.
";

// What the command line asked for.
#[derive(Serialize, Clone, Default)]
struct Start {
    file: Option<String>,
    edit: bool,
}

// Read the command line. Prints and exits for --help, --version and bad input.
fn parse_args(args: impl Iterator<Item = String>) -> Start {
    let mut start = Start::default();
    let mut flags_done = false;
    for arg in args {
        match arg.as_str() {
            "--" if !flags_done => flags_done = true,
            "-h" | "--help" if !flags_done => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "-v" | "-V" | "--version" if !flags_done => {
                println!("markinator {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-e" | "--edit" if !flags_done => start.edit = true,
            a if a.starts_with('-') && a.len() > 1 && !flags_done => {
                eprintln!("markinator: unknown option {a}\nTry: markinator --help");
                std::process::exit(2);
            }
            _ if start.file.is_none() => start.file = Some(arg),
            _ => {
                eprintln!("markinator: only one file can be opened\nTry: markinator --help");
                std::process::exit(2);
            }
        }
    }
    start
}

// Keeps the theme watcher alive for the whole run.
struct ThemeWatch(#[allow(dead_code)] Option<notify::RecommendedWatcher>);

// What each window should open at start, by window label.
struct Starts(Mutex<HashMap<String, Start>>);

#[tauri::command]
fn start(window: tauri::Window, starts: tauri::State<Starts>) -> Start {
    starts.0.lock().remove(window.label()).unwrap_or_default()
}

// Open a new window for `start`. Labels are reused ("main", "w2", "w3", ...)
// so the saved window size stays per slot and does not grow forever.
fn open_window(app: &tauri::AppHandle, start: Start) {
    let label = std::iter::once("main".to_string())
        .chain((2..).map(|n| format!("w{n}")))
        .find(|l| app.get_webview_window(l).is_none())
        .unwrap_or_default();
    app.state::<Starts>().0.lock().insert(label.clone(), start);
    let built = tauri::WebviewWindowBuilder::new(app, &label, tauri::WebviewUrl::default())
        .title("Untitled - markinator")
        .inner_size(900.0, 700.0)
        .decorations(false)
        .visible(false)
        .build();
    if let Err(e) = built {
        eprintln!("markinator: cannot open window: {e}");
    }
}

#[tauri::command]
fn theme() -> theme::Theme {
    theme::current()
}

// Three-button "Save changes?" dialog. Returns "save", "discard" or "cancel".
#[tauri::command]
async fn ask_save(app: tauri::AppHandle, window: tauri::Window) -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .message("Your changes will be lost if you don't save them.")
        .title("Save changes?")
        .kind(MessageDialogKind::Warning)
        .parent(&window)
        .buttons(MessageDialogButtons::YesNoCancelCustom(
            "Save".into(),
            "Don't save".into(),
            "Cancel".into(),
        ))
        .show_with_result(move |r| {
            let _ = tx.send(r);
        });
    let result = tauri::async_runtime::spawn_blocking(move || rx.recv()).await;
    // On Linux the plugin reports the button label as Custom.
    match result {
        Ok(Ok(MessageDialogResult::Yes)) => "save",
        Ok(Ok(MessageDialogResult::No)) => "discard",
        Ok(Ok(MessageDialogResult::Custom(s))) if s == "Save" => "save",
        Ok(Ok(MessageDialogResult::Custom(s))) if s == "Don't save" => "discard",
        _ => "cancel",
    }
    .to_string()
}

#[tauri::command]
fn show_error(app: tauri::AppHandle, text: String) {
    app.dialog()
        .message(text)
        .title("markinator")
        .kind(MessageDialogKind::Error)
        .blocking_show();
}

fn main() {
    let first = parse_args(std::env::args().skip(1));
    tauri::Builder::default()
        // A second `markinator file.md` hands the file to the running app
        // and exits. The running app opens it in a new window.
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            let mut start = parse_args(argv.into_iter().skip(1));
            start.file = start.file.map(|f| Path::new(&cwd).join(f).to_string_lossy().into_owned());
            open_window(app, start);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(files::FileWatch::new())
        .manage(Starts(Mutex::new(HashMap::new())))
        .setup(move |app| {
            app.manage(ThemeWatch(theme::watch(app.handle().clone())));
            open_window(app.handle(), first);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // The frontend decides if closing is ok (unsaved changes).
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.emit_to(window.label(), "close-requested", ());
            }
            WindowEvent::Destroyed => {
                window.state::<files::FileWatch>().forget(window.label());
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            start,
            theme,
            ask_save,
            show_error,
            files::load_file,
            files::save_file,
            files::render,
            files::watch_file,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run markinator");
}
