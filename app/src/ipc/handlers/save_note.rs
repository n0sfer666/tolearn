use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::notes::{roadmap, taken, view};
use crate::ipc::types::{SaveNoteIn, SaveNoteOut};
use crate::ipc::vaulted::{Written, store};

pub fn run(context: &Context, input: &SaveNoteIn) -> Result<SaveNoteOut, IpcError> {
    let written = store(context, input.directory.as_ref())?.save(
        &roadmap(&input.bundle)?,
        &input.topic,
        &input.body,
        input.stamp.as_ref().and_then(taken),
    )?;

    Ok(match written {
        Written::Saved(stamp) => SaveNoteOut {
            saved: true,
            stamp: Some(view(stamp)),
            theirs: None,
        },
        Written::Conflict(theirs) => SaveNoteOut {
            saved: false,
            stamp: None,
            theirs: Some(theirs),
        },
    })
}
