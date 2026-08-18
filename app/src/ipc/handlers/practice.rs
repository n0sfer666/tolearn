use tolearn_core::practice::{Session, Step, advance, timer};
use tolearn_core::progress::save;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{PracticeIn, PracticeOut};

pub fn run(_context: &Context, input: &PracticeIn) -> Result<PracticeOut, IpcError> {
    let mut opened = open::open(&input.bundle)?;
    let box_min = opened
        .scan
        .topics
        .iter()
        .find(|topic| topic.id == input.topic)
        .map(|topic| topic.practice.time_box_min)
        .ok_or_else(|| IpcError::unknown_topic(&input.topic))?;
    let session = opened
        .document
        .progress()
        .state(&input.topic)
        .map(|state| state.practice.clone())
        .unwrap_or_default();

    let session = match step(&input.step)? {
        None => session,
        Some(step) => {
            let moved = advance(&session, step, box_min, &input.now);
            opened.document.start(&input.topic)?;
            opened.document.practice(&input.topic, &moved)?;
            let file = open::progress_file(&opened.scan);
            save(&file, &opened.document)
                .map_err(|error| IpcError::unwritable(&file, &error.to_string()))?;
            moved
        }
    };

    Ok(view(&session, box_min, &input.now))
}

fn view(session: &Session, box_min: u32, now: &str) -> PracticeOut {
    let left = timer(session, box_min, now);
    PracticeOut {
        spent_sec: left.spent_sec,
        left_sec: left.left_sec,
        box_min,
        running: left.running,
        expired: left.expired,
    }
}

fn step(value: &str) -> Result<Option<Step>, IpcError> {
    match value {
        "peek" => Ok(None),
        "start" => Ok(Some(Step::Start)),
        "pause" => Ok(Some(Step::Pause)),
        "reset" => Ok(Some(Step::Reset)),
        other => Err(IpcError::new(
            "practice.step.unknown",
            format!("`{other}` — не шаг таймера практики"),
        )),
    }
}
