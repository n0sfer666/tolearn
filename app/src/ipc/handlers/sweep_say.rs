use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::sweeping::view;
use crate::ipc::types::{SweepSayIn, SweepStateOut};
use crate::sweep::answer;

pub fn run(context: &Context, input: &SweepSayIn) -> Result<SweepStateOut, IpcError> {
    if input.text.trim().is_empty() {
        return Err(IpcError::new(
            "sweep.empty-answer",
            "пустой ответ не отправляется".to_owned(),
        ));
    }
    Ok(view(&answer(
        context,
        &input.bundle,
        &input.today,
        &input.text,
    )?))
}
