use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::{roadmap, view};
use crate::ipc::types::{NoteIn, NoteOut};
use crate::ipc::vaulted::store;

pub fn run(context: &Context, input: &NoteIn) -> Result<NoteOut, IpcError> {
    let found =
        store(context, input.directory.as_ref())?.read(&roadmap(&input.bundle)?, &input.topic)?;

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
