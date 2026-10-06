mod script;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tolearn_offline::page::{PageError, Prerenderer};

use crate::hidden::{answer, on_main};

#[derive(Debug, Clone, Copy)]
pub struct Settling {
    pub settle: Duration,
    pub timeout: Duration,
    pub poll: Duration,
}

impl Default for Settling {
    fn default() -> Self {
        Self {
            settle: Duration::from_millis(500),
            timeout: Duration::from_secs(10),
            poll: Duration::from_millis(100),
        }
    }
}

#[derive(Debug)]
pub struct Webview<R: Runtime> {
    app: AppHandle<R>,
    settling: Settling,
}

static OPENED: AtomicUsize = AtomicUsize::new(0);

impl<R: Runtime> Webview<R> {
    pub fn new(app: AppHandle<R>, settling: Settling) -> Self {
        Self { app, settling }
    }

    fn open(&self, url: &str) -> Result<WebviewWindow<R>, PageError> {
        let address = url
            .parse()
            .map_err(|_| PageError::Prerender(format!("{url} — не адрес")))?;
        let label = format!("prerender-{}", OPENED.fetch_add(1, Ordering::Relaxed) + 1);
        let script = script::watcher(millis(self.settling.settle));
        let app = self.app.clone();
        on_main(&self.app, move || {
            WebviewWindowBuilder::new(&app, label, WebviewUrl::External(address))
                .visible(false)
                .focused(false)
                .skip_taskbar(true)
                .initialization_script(script)
                .build()
                .map_err(|error| error.to_string())
        })
        .flatten()
        .map_err(PageError::Prerender)
    }

    fn close(&self, window: WebviewWindow<R>) {
        let _ = on_main(&self.app, move || window.destroy());
    }

    fn watch(&self, window: &WebviewWindow<R>, until: Instant) -> Option<Vec<u8>> {
        while Instant::now() < until {
            if let Some(html) = answer(window, script::HARVEST, until) {
                return Some(html.into_bytes());
            }
            std::thread::sleep(self.settling.poll);
        }
        None
    }
}

impl<R: Runtime> Prerenderer for Webview<R> {
    fn render(&self, url: &str, html: &[u8]) -> Result<Vec<u8>, PageError> {
        let window = self.open(url)?;
        let rendered = self.watch(&window, Instant::now() + self.settling.timeout);
        self.close(window);
        Ok(rendered.unwrap_or_else(|| html.to_vec()))
    }
}

fn millis(span: Duration) -> u64 {
    u64::try_from(span.as_millis()).unwrap_or(u64::MAX)
}
