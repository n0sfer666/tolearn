use tolearn_core::review::{Past, Reviewed, review};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{PastView, ReviewIn, ReviewOut, ReviewedView};

pub fn run(_context: &Context, input: &ReviewIn) -> Result<ReviewOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let document = opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;
    let seen = review(document, opened.document.progress().state(&input.topic));

    Ok(ReviewOut {
        id: document.id.clone(),
        title: document.title.clone(),
        questions: seen.questions.iter().map(question).collect(),
        loose: seen.loose,
        last: seen.last.as_ref().map(past),
        history: seen.history.iter().map(past).collect(),
        split_suggested: seen.split_suggested,
        split_request: seen.split_request,
    })
}

fn question(source: &Reviewed) -> ReviewedView {
    ReviewedView {
        id: source.id.clone(),
        kind: source.kind.clone(),
        text: source.text.clone(),
        outcome: source.outcome.map(|outcome| outcome.label().to_owned()),
        missed: source.missed.clone(),
    }
}

fn past(source: &Past) -> PastView {
    PastView {
        at: source.at.clone(),
        verdict: source.verdict.label().to_owned(),
    }
}
