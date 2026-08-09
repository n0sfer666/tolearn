#[cfg(target_os = "macos")]
pub fn enable(app: &tauri::App) -> Result<(), tauri::Error> {
    use tauri::Manager as _;

    const WINDOW: &str = "main";

    let Some(webview) = app.get_webview_window(WINDOW) else {
        return Ok(());
    };
    webview.with_webview(|platform| tolearn_gestures::back_forward(platform.inner()))
}

#[cfg(not(target_os = "macos"))]
pub fn enable(_app: &tauri::App) -> Result<(), tauri::Error> {
    Ok(())
}
