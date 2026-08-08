use serde_json::json;
use tauri::{AppHandle, Manager as _, Url, WebviewWindow};

use crate::ipc;

const WINDOW: &str = "main";

pub fn raised(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WINDOW) else {
        return;
    };
    let _ = window.unminimize();
    let _ = window.set_focus();
}

pub fn opened(app: &AppHandle, urls: &[Url]) {
    let Some(url) = urls.first() else {
        return;
    };
    let Some(window) = app.get_webview_window(WINDOW) else {
        return;
    };
    if let Some(target) = addressed(&window, &asked(app, url.as_str())) {
        let _ = window.navigate(target);
    }
    raised(app);
}

fn asked(app: &AppHandle, url: &str) -> Vec<(String, String)> {
    let answer =
        ipc::of(app).and_then(|context| ipc::call(&context, "follow", &json!({ "url": url })));
    match answer {
        Ok(out) => vec![
            ("program".to_owned(), taken(&out, "program")),
            ("topic".to_owned(), taken(&out, "topic")),
        ],
        Err(error) => vec![("refused".to_owned(), error.message)],
    }
}

fn taken(out: &serde_json::Value, name: &str) -> String {
    out[name].as_str().unwrap_or_default().to_owned()
}

fn addressed(window: &WebviewWindow, pairs: &[(String, String)]) -> Option<Url> {
    let mut target = window.url().ok()?.join("/").ok()?;
    let mut query = target.query_pairs_mut();
    for (name, value) in pairs {
        query.append_pair(name, value);
    }
    drop(query);
    Some(target)
}
