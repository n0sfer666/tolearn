use tolearn_core::Date;
use tolearn_core::status::effective;
use tolearn_core::summary::{self, Summary};

use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{ProgramIn, ProgramOut, Span, Stage, Tally, TopicStatus};

pub fn run(input: &ProgramIn) -> Result<ProgramOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let day = Date::parse(&input.today).ok_or_else(|| IpcError::malformed_date(&input.today))?;
    let statuses = effective(
        &opened.scan.roadmap,
        &opened.scan.topics,
        opened.document.progress(),
        day,
    );
    let summary = summary::summarize(&opened.scan.roadmap, &opened.scan.topics, &statuses);

    Ok(ProgramOut {
        program: tally(&summary.program),
        stages: stages(&summary),
        topics: statuses
            .iter()
            .map(|(id, status)| TopicStatus {
                id: id.clone(),
                status: status.label().to_owned(),
            })
            .collect(),
    })
}

fn stages(summary: &Summary) -> Vec<Stage> {
    summary
        .stages
        .iter()
        .map(|stage| Stage {
            n: stage.n,
            title: stage.title.clone(),
            tally: tally(&stage.tally),
        })
        .collect()
}

fn tally(counted: &summary::Tally) -> Tally {
    Tally {
        done: counted.done,
        total: counted.total,
        stale: counted.stale,
        share: counted.share(),
        hours_done: Span {
            min: counted.hours_done.min,
            max: counted.hours_done.max,
        },
        hours_total: Span {
            min: counted.hours_total.min,
            max: counted.hours_total.max,
        },
    }
}
