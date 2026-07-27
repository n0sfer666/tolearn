use std::path::PathBuf;

use tolearn_core::notes::{NoteError, Stamp};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::StampView;

pub fn roadmap(bundle: &str) -> Result<String, IpcError> {
    Ok(crate::ipc::open::read(bundle)?.roadmap.id)
}

pub fn root(context: &Context, directory: Option<&String>) -> PathBuf {
    directory.map_or_else(|| context.notes(), PathBuf::from)
}

pub fn view(stamp: Stamp) -> StampView {
    StampView {
        modified_nanos: stamp.modified_nanos.to_string(),
        size: stamp.size,
    }
}

pub fn taken(view: &StampView) -> Option<Stamp> {
    Some(Stamp {
        modified_nanos: view.modified_nanos.parse().ok()?,
        size: view.size,
    })
}

pub fn failed(error: NoteError) -> IpcError {
    let code = match error {
        NoteError::Unreadable { .. } => "note.unreadable",
        NoteError::Unwritable { .. } => "note.unwritable",
        NoteError::Conflict { .. } => "note.conflict",
    };
    IpcError::new(code, error.to_string())
}
