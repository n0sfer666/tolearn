pub mod generate;
pub mod gestures;
pub mod ipc;
pub mod journal;
pub mod offline;
pub mod prerender;
pub mod speech;
pub mod window;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            window::raised(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            offline::install(Box::new(prerender::Webview::new(
                app.handle().clone(),
                prerender::Settling::default(),
            )));
            gestures::enable(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ipc::contract::command])
        .run(tauri::generate_context!())
}
