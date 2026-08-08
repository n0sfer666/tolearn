use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::sweeping::view;
use crate::ipc::types::{SweepStartIn, SweepStateOut};
use crate::sweep::start;

pub fn run(context: &Context, input: &SweepStartIn) -> Result<SweepStateOut, IpcError> {
    Ok(view(&start(
        context,
        &input.bundle,
        &input.today,
        input.topics as usize,
        input.restart,
    )?))
}
