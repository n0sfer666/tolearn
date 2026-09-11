pub mod gestures;
pub mod ipc;
pub mod journal;
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
            gestures::enable(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ipc::contract::command])
        .run(tauri::generate_context!())
}
