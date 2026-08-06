use tolearn_core::prompt::render;
use tolearn_provider::{Provider, ask};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::provider::{denied, failed, refute};
use crate::ipc::types::{ExamineIn, ExamineOut};

pub fn run(context: &Context, input: &ExamineIn) -> Result<ExamineOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let topic = scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;
    let template = open::text(&scan.root.join("examiner.md"))?;
    let prompt = render(&template, topic, &scan.roadmap)?;

    let provider = Provider::read(&context.provider()).map_err(failed)?;
    let key = context.vault().key().map_err(denied)?;
    Ok(ExamineOut {
        text: ask(&provider, key.as_deref(), &prompt)
            .map_err(refute)?
            .text,
    })
}
