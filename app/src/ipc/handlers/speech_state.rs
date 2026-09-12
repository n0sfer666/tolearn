use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{SpeechStateIn, SpeechStateOut};
use crate::speech;

pub fn run(context: &Context, input: &SpeechStateIn) -> Result<SpeechStateOut, IpcError> {
    let tree = context.library().open(&input.program)?;
    Ok(SpeechStateOut {
        available: speech::available(),
        listening: speech::listening(),
        language: speech::tongue(&tree.program.generation.locale),
    })
}
