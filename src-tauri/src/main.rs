// markinator: write markdown, press Ctrl+E, read it.

mod files;
mod theme;

use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_cli::CliExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind, MessageDialogResult};

// File given on the command line, if any.
struct StartFile(Option<String>);

// Keeps the theme watcher alive for the whole run.
struct ThemeWatch(#[allow(dead_code)] Option<notify::RecommendedWatcher>);

#[tauri::command]
fn start_file(state: tauri::State<StartFile>) -> Option<String> {
    state.0.clone()
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
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(files::FileWatch::new())
        .setup(|app| {
            let file = app
                .cli()
                .matches()
                .ok()
                .and_then(|m| m.args.get("file").and_then(|a| a.value.as_str().map(String::from)));
            app.manage(StartFile(file));
            app.manage(ThemeWatch(theme::watch(app.handle().clone())));
            Ok(())
        })
        .on_window_event(|window, event| {
            // The frontend decides if closing is ok (unsaved changes).
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.emit("close-requested", ());
            }
        })
        .invoke_handler(tauri::generate_handler![
            start_file,
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
