use tolearn_core::scan::Scan;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::types::{SaveOfflineIn, SaveOfflineOut};
use crate::ipc::{open, settings};
use crate::offline::{self, Mode, Piece};

pub fn run(context: &Context, input: &SaveOfflineIn) -> Result<SaveOfflineOut, IpcError> {
    let scan = open::read(&input.bundle)?;
    let mut pieces = wanted(&scan, input.topic.as_deref())?;
    let settings = settings::stored(context)?;
    let budget = settings.budget_bytes();

    let mode = match input.again.as_deref() {
        Some(previous) => {
            let broken = offline::failed(previous);
            pieces.retain(|piece| broken.contains(&piece.material.url));
            Mode::Save
        }
        None => Mode::Refresh,
    };

    let total = u32::try_from(pieces.len()).unwrap_or(u32::MAX);
    let job = offline::start(&context.offline(), budget, &scan.roadmap.id, pieces, mode);
    Ok(SaveOfflineOut { job, total })
}

fn wanted(scan: &Scan, topic: Option<&str>) -> Result<Vec<Piece>, IpcError> {
    let Some(topic) = topic else {
        return Ok(offline::every(scan));
    };
    offline::of(scan, topic).ok_or_else(|| IpcError::unknown_topic(topic))
}
