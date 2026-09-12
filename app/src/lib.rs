pub mod gestures;
mod hidden;
pub mod ipc;
pub mod journal;
pub mod mermaid;
pub mod prerender;
pub mod speech;
pub mod window;

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            window::raised(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(mermaid::plugin())
        .manage(ipc::Running::default())
        .manage(ipc::Ledger::default())
        .setup(|app| {
            gestures::enable(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ipc::contract::command])
        .run(tauri::generate_context!())
}
