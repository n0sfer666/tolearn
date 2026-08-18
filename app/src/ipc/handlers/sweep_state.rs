use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::sweeping::view;
use crate::ipc::types::{SweepStateIn, SweepStateOut};
use crate::sweep::look;

pub fn run(context: &Context, input: &SweepStateIn) -> Result<SweepStateOut, IpcError> {
    Ok(view(&look(context, &input.bundle, &input.today)?))
}
