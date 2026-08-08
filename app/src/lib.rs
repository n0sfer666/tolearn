pub mod exam;
pub mod generate;
pub mod ipc;
pub mod journal;
pub mod link;
pub mod offline;
pub mod prerender;
pub mod speech;
pub mod sweep;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            link::raised(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager as _;
            use tauri_plugin_deep_link::DeepLinkExt as _;

            let handle = app.handle().clone();
            app.deep_link()
                .on_open_url(move |event| link::opened(&handle, &event.urls()));
            offline::install(Box::new(prerender::Webview::new(
                app.handle().clone(),
                prerender::Settling::default(),
            )));
            if let Some(webview) = app.get_webview_window("main") {
                webview
                    .with_webview(|platform| tolearn_gestures::back_forward(platform.inner()))?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ipc::contract::command])
        .run(tauri::generate_context!())
}
