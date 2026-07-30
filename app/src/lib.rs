pub mod ipc;
pub mod prerender;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager as _;

            if let Some(webview) = app.get_webview_window("main") {
                webview
                    .with_webview(|platform| tolearn_gestures::back_forward(platform.inner()))?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ipc::contract::command])
        .run(tauri::generate_context!())
}
