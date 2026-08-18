//! Включает нативный жест «назад/вперёд» у платформенного вебвью.
//!
//! Единственный крейт, которому разрешён `unsafe`: обращение к WKWebView идёт
//! по сырому указателю, который отдаёт Tauri. На остальных системах вызов
//! ничего не делает — жеста там нет.

/// Указатель на платформенное вебвью: на macOS это `WKWebView`.
///
/// Вызывающий обязан передать указатель, полученный от рантайма прямо сейчас,
/// и делать это на главном потоке.
#[cfg(target_os = "macos")]
pub fn back_forward(webview: *mut std::ffi::c_void) {
    use objc2::rc::Retained;
    use objc2_web_kit::WKWebView;

    if webview.is_null() {
        return;
    }
    let view = webview.cast::<WKWebView>();
    unsafe {
        let view = Retained::retain(view);
        if let Some(view) = view {
            view.setAllowsBackForwardNavigationGestures(true);
        }
    }
}

/// На системах без нативного жеста включать нечего.
#[cfg(not(target_os = "macos"))]
pub fn back_forward(_webview: *mut std::ffi::c_void) {}
