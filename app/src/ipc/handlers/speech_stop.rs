use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{SpeechStopIn, SpeechStopOut};
use crate::speech;

pub fn run(context: &Context, input: &SpeechStopIn) -> Result<SpeechStopOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let text = speech::stop(context.resources(), &speech::tongue(&scan.roadmap.locale))?;
    Ok(SpeechStopOut { text })
}
