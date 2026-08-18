use std::path::PathBuf;

use tolearn_core::atomic;

use crate::exam::safe;
use crate::ipc::Context;
use crate::ipc::IpcError;

use super::run::Run;

pub fn load(context: &Context, roadmap: &str) -> Option<Run> {
    let text = std::fs::read_to_string(file(context, roadmap)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(context: &Context, run: &Run) -> Result<(), IpcError> {
    let path = file(context, &run.roadmap);
    if let Some(room) = path.parent() {
        std::fs::create_dir_all(room)
            .map_err(|error| IpcError::unwritable(room, &error.to_string()))?;
    }
    let text = serde_json::to_string_pretty(run).map_err(|error| IpcError::payload(&error))?;
    atomic::write(&path, &text).map_err(|error| IpcError::unwritable(&path, &error.to_string()))
}

pub fn forget(context: &Context, roadmap: &str) {
    let _ = std::fs::remove_file(file(context, roadmap));
}

fn file(context: &Context, roadmap: &str) -> PathBuf {
    context.sweeps().join(format!("{}.json", safe(roadmap)))
}
