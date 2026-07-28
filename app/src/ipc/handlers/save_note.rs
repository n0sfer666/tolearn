use tolearn_core::notes::{NoteError, save};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::{failed, roadmap, root, taken, view};
use crate::ipc::types::{SaveNoteIn, SaveNoteOut};

pub fn run(context: &Context, input: &SaveNoteIn) -> Result<SaveNoteOut, IpcError> {
    let written = save(
        &root(context, input.directory.as_ref())?,
        &roadmap(&input.bundle)?,
        &input.topic,
        &input.body,
        input.stamp.as_ref().and_then(taken),
    );

    match written {
        Ok(stamp) => Ok(SaveNoteOut {
            saved: true,
            stamp: Some(view(stamp)),
            theirs: None,
        }),
        Err(NoteError::Conflict { theirs, .. }) => Ok(SaveNoteOut {
            saved: false,
            stamp: None,
            theirs: Some(theirs),
        }),
        Err(error) => Err(failed(error)),
    }
}
