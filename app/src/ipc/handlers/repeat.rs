use tolearn_core::Date;
use tolearn_core::progress::save;
use tolearn_core::queue::repeat;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{RepeatIn, RepeatOut};

pub fn run(_context: &Context, input: &RepeatIn) -> Result<RepeatOut, IpcError> {
    let mut opened = open::open(&input.bundle)?;
    let today = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let topic = opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;

    let Some(moved) = repeat(topic, today) else {
        return Ok(RepeatOut {
            next_review_at: None,
        });
    };
    let moved = moved.to_string();
    opened.document.reschedule(&input.topic, &moved)?;

    let file = open::progress_file(&opened.scan);
    save(&file, &opened.document)
        .map_err(|error| IpcError::unwritable(&file, &error.to_string()))?;

    Ok(RepeatOut {
        next_review_at: Some(moved),
    })
}
