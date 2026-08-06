use crate::exam::start;
use crate::ipc::context::Context;
use crate::ipc::dialogue::view;
use crate::ipc::error::IpcError;
use crate::ipc::types::{ExamStartIn, ExamStateOut};

pub fn run(context: &Context, input: &ExamStartIn) -> Result<ExamStateOut, IpcError> {
    Ok(view(&start(
        context,
        &input.bundle,
        &input.topic,
        input.restart,
    )?))
}
