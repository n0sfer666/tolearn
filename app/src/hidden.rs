use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Instant;

use tauri::{AppHandle, Runtime, WebviewWindow};

pub(crate) fn on_main<R: Runtime, T: Send + 'static>(
    app: &AppHandle<R>,
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    let (done, result) = mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = done.send(work());
    })
    .map_err(|error| error.to_string())?;
    result
        .recv()
        .map_err(|_| "окно не открылось: цикл событий молчит".to_owned())
}

pub(crate) fn answer<R: Runtime>(
    window: &WebviewWindow<R>,
    script: &str,
    until: Instant,
) -> Option<String> {
    let (answers, answer) = mpsc::channel();
    window
        .eval_with_callback(script, move |json| {
            let _ = answers.send(json);
        })
        .ok()?;
    let left = until.saturating_duration_since(Instant::now());
    match answer.recv_timeout(left) {
        Ok(json) => serde_json::from_str::<Option<String>>(&json).ok().flatten(),
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => None,
    }
}
