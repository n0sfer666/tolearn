use tolearn_core::Date;

use crate::exam::finish;
use crate::ipc::context::Context;
use crate::ipc::dialogue::view;
use crate::ipc::error::IpcError;
use crate::ipc::types::{ExamFinishIn, ExamStateOut};

pub fn run(context: &Context, input: &ExamFinishIn) -> Result<ExamStateOut, IpcError> {
    Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    Ok(view(&finish(
        context,
        &input.bundle,
        &input.topic,
        &input.today,
    )?))
}
