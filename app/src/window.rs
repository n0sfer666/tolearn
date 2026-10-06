use tauri::{AppHandle, Manager as _};

const WINDOW: &str = "main";

pub fn raised(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WINDOW) else {
        return;
    };
    let _ = window.unminimize();
    let _ = window.set_focus();
}
