use tolearn_core::Date;
use tolearn_core::roadmap::Roadmap;
use tolearn_core::status::{Statuses, effective, is_done};
use tolearn_core::summary::{self, Summary};
use tolearn_core::topic::Topic;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::open;
use crate::ipc::types::{Link, ProgramIn, ProgramOut, Span, Stage, Tally, TopicStatus};

pub fn run(_context: &Context, input: &ProgramIn) -> Result<ProgramOut, IpcError> {
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
        stages: stages(&opened.scan.roadmap, &summary),
        topics: topics(&opened.scan.roadmap, &opened.scan.topics, &statuses),
    })
}

fn stages(roadmap: &Roadmap, summary: &Summary) -> Vec<Stage> {
    summary
        .stages
        .iter()
        .map(|stage| Stage {
            n: stage.n,
            title: stage.title.clone(),
            checkpoint: checkpoint_of(roadmap, stage.n),
            tally: tally(&stage.tally),
        })
        .collect()
}

fn checkpoint_of(roadmap: &Roadmap, stage: u32) -> String {
    roadmap
        .stages
        .iter()
        .find(|entry| entry.n == stage)
        .map(|entry| entry.checkpoint.clone())
        .unwrap_or_default()
}

fn topics(roadmap: &Roadmap, documents: &[Topic], statuses: &Statuses) -> Vec<TopicStatus> {
    roadmap
        .topics
        .iter()
        .map(|entry| TopicStatus {
            id: entry.id.clone(),
            title: entry.title.clone(),
            stage: entry.stage,
            checkpoint: roadmap
                .stages
                .iter()
                .any(|stage| stage.checkpoint == entry.id),
            status: statuses
                .get(&entry.id)
                .map(|status| status.label().to_owned())
                .unwrap_or_default(),
            hours: Span {
                min: entry.est_hours.min,
                max: entry.est_hours.max,
            },
            blocked_by: blocked_by(roadmap, documents, statuses, &entry.id),
        })
        .collect()
}

fn blocked_by(roadmap: &Roadmap, documents: &[Topic], statuses: &Statuses, id: &str) -> Vec<Link> {
    let Some(document) = documents.iter().find(|topic| topic.id == id) else {
        return Vec::new();
    };
    document
        .depends_on
        .iter()
        .filter(|needed| !statuses.get(needed).is_some_and(is_done))
        .map(|needed| Link {
            id: needed.clone(),
            title: title_of(roadmap, needed),
        })
        .collect()
}

fn title_of(roadmap: &Roadmap, id: &str) -> String {
    roadmap
        .topics
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.title.clone())
        .unwrap_or_else(|| id.to_owned())
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
