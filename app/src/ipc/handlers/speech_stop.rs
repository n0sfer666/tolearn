use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{SpeechStopIn, SpeechStopOut};
use crate::speech;

pub fn run(context: &Context, input: &SpeechStopIn) -> Result<SpeechStopOut, IpcError> {
    let tree = context
        .library()
        .open(&input.program)
        .inspect_err(|_| speech::cancel())?;
    let text = speech::stop(
        context.resources(),
        &speech::tongue(&tree.program.generation.locale),
    )?;
    Ok(SpeechStopOut { text })
}
