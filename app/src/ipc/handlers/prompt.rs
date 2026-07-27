use tolearn_core::prompt::render;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{PromptIn, PromptOut};

pub fn run(_context: &Context, input: &PromptIn) -> Result<PromptOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let topic = scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;
    let template = open::text(&scan.root.join("examiner.md"))?;
    Ok(PromptOut {
        text: render(&template, topic, &scan.roadmap)?,
    })
}
