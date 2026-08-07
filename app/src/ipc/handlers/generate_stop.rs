use crate::generate::{stop, unpin};
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{GenerateStopIn, GenerateStopOut};

pub fn run(context: &Context, input: &GenerateStopIn) -> Result<GenerateStopOut, IpcError> {
    let stopping = stop(&input.job);
    unpin(&context.draft());
    Ok(GenerateStopOut { stopping })
}
