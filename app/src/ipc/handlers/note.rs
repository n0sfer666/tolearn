use tolearn_core::notes::read;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::{failed, roadmap, root, view};
use crate::ipc::types::{NoteIn, NoteOut};

pub fn run(context: &Context, input: &NoteIn) -> Result<NoteOut, IpcError> {
    let found = read(
        &root(context, input.directory.as_ref())?,
        &roadmap(&input.bundle)?,
        &input.topic,
    )
    .map_err(failed)?;

    Ok(match found {
        None => NoteOut {
            body: String::new(),
            path: None,
            stamp: None,
        },
        Some(note) => NoteOut {
            body: note.body,
            path: Some(note.path.display().to_string()),
            stamp: Some(view(note.stamp)),
        },
    })
}
