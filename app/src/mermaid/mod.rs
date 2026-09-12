mod page;
mod protocol;

pub use protocol::{POLICY, SCHEME, plugin, respond};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;
use tauri::{AppHandle, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tolearn_generate::diagram::Painter;

use crate::hidden::{answer, on_main};

const HARVEST: &str = "window.__tolearn === undefined ? null : JSON.stringify(window.__tolearn)";

#[derive(Debug, Clone, Copy)]
pub struct Drawing {
    pub timeout: Duration,
    pub poll: Duration,
}

impl Default for Drawing {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            poll: Duration::from_millis(100),
        }
    }
}

#[derive(Debug)]
pub struct Mermaid<R: Runtime> {
    app: AppHandle<R>,
    drawing: Drawing,
}

#[derive(Deserialize)]
struct Answer {
    svg: Option<String>,
    error: Option<String>,
}

static OPENED: AtomicUsize = AtomicUsize::new(0);

impl<R: Runtime> Mermaid<R> {
    pub fn new(app: AppHandle<R>, drawing: Drawing) -> Self {
        Self { app, drawing }
    }

    fn open(&self, mermaid: &str) -> Result<WebviewWindow<R>, String> {
        let address = protocol::address(mermaid)?;
        let label = format!("mermaid-{}", OPENED.fetch_add(1, Ordering::Relaxed) + 1);
        let app = self.app.clone();
        on_main(&self.app, move || {
            WebviewWindowBuilder::new(&app, label, WebviewUrl::CustomProtocol(address))
                .visible(false)
                .focused(false)
                .skip_taskbar(true)
                .on_navigation(protocol::inside)
                .build()
                .map_err(|error| error.to_string())
        })?
    }

    fn watch(&self, window: &WebviewWindow<R>) -> Result<String, String> {
        let until = Instant::now() + self.drawing.timeout;
        while Instant::now() < until {
            if let Some(json) = answer(window, HARVEST, until) {
                return parsed(&json);
            }
            std::thread::sleep(self.drawing.poll);
        }
        Err(format!(
            "схема не нарисовалась за {} мс",
            self.drawing.timeout.as_millis()
        ))
    }
}

impl<R: Runtime> Painter for Mermaid<R> {
    fn paint(&self, mermaid: &str) -> Result<String, String> {
        let window = self.open(mermaid)?;
        let drawn = self.watch(&window);
        let _ = on_main(&self.app, move || window.destroy());
        drawn
    }
}

fn parsed(json: &str) -> Result<String, String> {
    match serde_json::from_str::<Answer>(json) {
        Ok(Answer { svg: Some(svg), .. }) => Ok(svg),
        Ok(Answer {
            error: Some(error), ..
        }) => Err(format!("Mermaid не разобрал схему: {error}")),
        Ok(_) => Err("окно схемы ответило пустым".to_owned()),
        Err(error) => Err(format!("окно схемы ответило не по формату: {error}")),
    }
}
