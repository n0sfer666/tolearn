use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::sweeping::settled;
use crate::ipc::types::{SweepAcceptOut, SweepStateIn};
use crate::sweep::accept;

pub fn run(context: &Context, input: &SweepStateIn) -> Result<SweepAcceptOut, IpcError> {
    Ok(SweepAcceptOut {
        settled: settled(&accept(context, &input.bundle, &input.today)?),
    })
}
