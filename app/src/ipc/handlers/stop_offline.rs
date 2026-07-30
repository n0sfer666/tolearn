use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{StopOfflineIn, StopOfflineOut};
use crate::offline;

pub fn run(_context: &Context, input: &StopOfflineIn) -> Result<StopOfflineOut, IpcError> {
    Ok(StopOfflineOut {
        stopping: offline::stop(&input.job),
    })
}
