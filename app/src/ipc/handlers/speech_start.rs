use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{SpeechStateIn, SpeechStateOut};
use crate::speech;

pub fn run(_context: &Context, input: &SpeechStateIn) -> Result<SpeechStateOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    speech::start()?;
    Ok(SpeechStateOut {
        available: speech::available(),
        listening: speech::listening(),
        language: speech::tongue(&scan.roadmap.locale),
    })
}
