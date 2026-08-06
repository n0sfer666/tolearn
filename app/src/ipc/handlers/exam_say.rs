use crate::exam::answer;
use crate::ipc::context::Context;
use crate::ipc::dialogue::view;
use crate::ipc::error::IpcError;
use crate::ipc::types::{ExamSayIn, ExamStateOut};

pub fn run(context: &Context, input: &ExamSayIn) -> Result<ExamStateOut, IpcError> {
    if input.text.trim().is_empty() {
        return Err(IpcError::new(
            "exam.empty-answer",
            "пустой ответ не отправляется".to_owned(),
        ));
    }
    Ok(view(&answer(
        context,
        &input.bundle,
        &input.topic,
        &input.text,
    )?))
}
