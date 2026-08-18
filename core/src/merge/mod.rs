mod essence;
mod report;

pub use report::{Part, Report, Stale};

pub(crate) use essence::changed;

use std::collections::BTreeSet;

use crate::progress::{Document, DocumentError, Status};
use crate::roadmap::Roadmap;
use crate::topic::Topic;

pub fn merge(
    document: &mut Document,
    roadmap: &Roadmap,
    before: &[Topic],
    after: &[Topic],
) -> Result<Report, DocumentError> {
    let mut report = Report::default();

    for entry in &roadmap.topics {
        if document.progress().state(&entry.id).is_none() {
            document.start(&entry.id)?;
            report.added.push(entry.id.clone());
            continue;
        }
        report.kept.push(entry.id.clone());

        let (Some(was), Some(is)) = (find(before, &entry.id), find(after, &entry.id)) else {
            continue;
        };
        let changed = essence::changed(was, is);
        if changed.is_empty() {
            continue;
        }
        if status(document, &entry.id) == Some(Status::Passed) {
            document.restate(&entry.id, Status::StalePassed)?;
            report.stale.push(Stale {
                id: entry.id.clone(),
                changed,
            });
        }
    }

    let known: BTreeSet<&str> = roadmap
        .topics
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    report.orphaned = document
        .progress()
        .topics
        .iter()
        .map(|(id, _)| id)
        .filter(|id| !known.contains(id.as_str()))
        .cloned()
        .collect();

    Ok(report)
}

fn find<'a>(topics: &'a [Topic], id: &str) -> Option<&'a Topic> {
    topics.iter().find(|topic| topic.id == id)
}

fn status(document: &Document, id: &str) -> Option<Status> {
    document.progress().state(id).map(|state| state.status)
}
