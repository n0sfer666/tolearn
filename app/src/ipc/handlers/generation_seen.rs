use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::generating::{GenerationStateIn, GenerationStateOut};

pub fn run(context: &Context, _input: &GenerationStateIn) -> Result<GenerationStateOut, IpcError> {
    Ok(context.running().seen())
}
