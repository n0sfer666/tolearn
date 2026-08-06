use std::path::PathBuf;

use tolearn_core::atomic;

use crate::ipc::Context;
use crate::ipc::IpcError;

use super::dialog::Dialog;

pub fn load(context: &Context, roadmap: &str, topic: &str) -> Option<Dialog> {
    let text = std::fs::read_to_string(file(context, roadmap, topic)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(context: &Context, roadmap: &str, dialog: &Dialog) -> Result<(), IpcError> {
    let path = file(context, roadmap, &dialog.topic);
    if let Some(room) = path.parent() {
        std::fs::create_dir_all(room)
            .map_err(|error| IpcError::unwritable(room, &error.to_string()))?;
    }
    let text = serde_json::to_string_pretty(dialog).map_err(|error| IpcError::payload(&error))?;
    atomic::write(&path, &text).map_err(|error| IpcError::unwritable(&path, &error.to_string()))
}

pub fn forget(context: &Context, roadmap: &str, topic: &str) {
    let _ = std::fs::remove_file(file(context, roadmap, topic));
}

fn file(context: &Context, roadmap: &str, topic: &str) -> PathBuf {
    context
        .dialogs()
        .join(safe(roadmap))
        .join(format!("{}.json", safe(topic)))
}

fn safe(name: &str) -> String {
    name.chars()
        .map(|symbol| match symbol {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => symbol,
            _ => '-',
        })
        .collect()
}
