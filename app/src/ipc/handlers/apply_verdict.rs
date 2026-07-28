use tolearn_core::Date;
use tolearn_core::progress::save;
use tolearn_core::protocol::{apply, record};
use tolearn_core::status::effective;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{ApplyVerdictIn, ApplyVerdictOut};
use crate::ipc::verdict::{of, read};

pub fn run(_context: &Context, input: &ApplyVerdictIn) -> Result<ApplyVerdictOut, IpcError> {
    let mut opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let topic = read(&opened, &input.topic)?;
    let verdict = of(&input.text, topic)?;

    let applied = apply(
        &verdict,
        topic,
        opened
            .document
            .progress()
            .topics
            .iter()
            .find(|(id, _)| *id == input.topic)
            .map(|(_, state)| state),
        &format!("{day}T00:00:00Z"),
        day,
    );
    record(&mut opened.document, &input.topic, &applied)?;

    let file = open::progress_file(&opened.scan);
    save(&file, &opened.document)
        .map_err(|error| IpcError::unwritable(&file, &error.to_string()))?;

    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    Ok(ApplyVerdictOut {
        status: statuses
            .get(&input.topic)
            .map(|status| status.label().to_owned())
            .unwrap_or_default(),
        gaps: applied.state.gaps.clone(),
        retry: applied.retry.clone(),
        split_suggested: applied.split_suggested,
    })
}
