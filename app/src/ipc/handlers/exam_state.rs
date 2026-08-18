use crate::exam::look;
use crate::ipc::context::Context;
use crate::ipc::dialogue::view;
use crate::ipc::error::IpcError;
use crate::ipc::types::{ExamStateIn, ExamStateOut};

pub fn run(context: &Context, input: &ExamStateIn) -> Result<ExamStateOut, IpcError> {
    Ok(view(&look(context, &input.bundle, &input.topic)?))
}
