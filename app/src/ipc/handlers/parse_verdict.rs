use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{ParseVerdictIn, VerdictView};
use crate::ipc::verdict::{of, read, view};

pub fn run(_context: &Context, input: &ParseVerdictIn) -> Result<VerdictView, IpcError> {
    let opened = open::open(&input.bundle)?;
    let topic = read(&opened, &input.topic)?;
    Ok(view(&of(&input.text, topic)?))
}
