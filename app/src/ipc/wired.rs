use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use crate::mermaid::{Drawing, Mermaid};
use crate::prerender::{Settling, Webview};

use super::context::Context;
use super::error::IpcError;
use super::running::Running;
use super::started::{GenerationStep, STEP_EVENT};
use super::tools::Tools;

pub fn wired(app: &AppHandle, context: Context) -> Result<Context, IpcError> {
    let emitter = app.clone();
    let tools = Tools {
        source: None,
        renderer: Some(Arc::new(Webview::new(app.clone(), Settling::default()))),
        painter: Some(Arc::new(Mermaid::new(app.clone(), Drawing::default()))),
        herald: Some(Arc::new(move |step: GenerationStep| {
            let _ = emitter.emit(STEP_EVENT, step);
        })),
    };
    let running = app
        .try_state::<Running>()
        .map(|state| state.inner().clone())
        .ok_or_else(|| {
            IpcError::new(
                "ipc.unmanaged",
                "в приложении не заведено состояние генерации".to_owned(),
            )
        })?;
    Ok(context.with_tools(tools).with_running(running))
}
