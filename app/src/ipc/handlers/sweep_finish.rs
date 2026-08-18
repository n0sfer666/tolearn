use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::sweeping::view;
use crate::ipc::types::{SweepStateIn, SweepStateOut};
use crate::sweep::finish;

pub fn run(context: &Context, input: &SweepStateIn) -> Result<SweepStateOut, IpcError> {
    Ok(view(&finish(context, &input.bundle, &input.today)?))
}
