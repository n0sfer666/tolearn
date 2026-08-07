use crate::generate::stop;
use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{GenerateStopIn, GenerateStopOut};

pub fn run(_context: &Context, input: &GenerateStopIn) -> Result<GenerateStopOut, IpcError> {
    Ok(GenerateStopOut {
        stopping: stop(&input.job),
    })
}
