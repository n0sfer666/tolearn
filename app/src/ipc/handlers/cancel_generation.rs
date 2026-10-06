use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::started::{CancelGenerationIn, CancelGenerationOut};

pub fn run(
    context: &Context,
    _input: &CancelGenerationIn,
) -> Result<CancelGenerationOut, IpcError> {
    Ok(CancelGenerationOut {
        cancelled: context.running().cancel(),
    })
}
