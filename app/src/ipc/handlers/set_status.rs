use tolearn_core::Date;
use tolearn_core::progress::{Status, save};
use tolearn_core::status::{effective, manual};

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{SetStatusIn, SetStatusOut};

pub fn run(_context: &Context, input: &SetStatusIn) -> Result<SetStatusOut, IpcError> {
    let mut opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let status = Status::parse(&input.status).ok_or_else(|| unknown_status(&input.status))?;
    let document = opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;

    let mark = manual(status, &format!("{day}T00:00:00Z"), day, document);
    opened.document.mark(&input.topic, &mark)?;

    let file = open::progress_file(&opened.scan);
    save(&file, &opened.document)
        .map_err(|error| IpcError::unwritable(&file, &error.to_string()))?;

    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    Ok(SetStatusOut {
        status: statuses
            .get(&input.topic)
            .map(|status| status.label().to_owned())
            .unwrap_or_default(),
    })
}

fn unknown_status(value: &str) -> IpcError {
    IpcError::new(
        "status.unknown",
        format!("`{value}` — не статус из протокола"),
    )
}
