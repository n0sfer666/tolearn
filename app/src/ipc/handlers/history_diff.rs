use tolearn_core::history::{Rewritten, Snapshot, diff};
use tolearn_core::roadmap::Roadmap;

use crate::ipc::context::Context;
use crate::ipc::error::IpcError;
use crate::ipc::history::room;
use crate::ipc::open;
use crate::ipc::types::{HistoryDiffIn, HistoryDiffOut, Link, RewrittenView};

pub fn run(context: &Context, input: &HistoryDiffIn) -> Result<HistoryDiffOut, IpcError> {
    let opened = open::open(&input.bundle)?;
    let store = context.history(&opened.scan.roadmap.id);
    let room = room(&store, input.version)
        .ok_or_else(|| IpcError::unknown_version(input.version, &opened.scan.roadmap.id))?;
    let kept = open::read(&room.display().to_string())?;

    let taken = diff(
        Snapshot {
            roadmap: &kept.roadmap,
            topics: &kept.topics,
        },
        Snapshot {
            roadmap: &opened.scan.roadmap,
            topics: &opened.scan.topics,
        },
        opened.document.progress(),
    );

    Ok(HistoryDiffOut {
        added: taken
            .added
            .iter()
            .map(|id| link(&opened.scan.roadmap, id))
            .collect(),
        removed: taken
            .removed
            .iter()
            .map(|id| link(&kept.roadmap, id))
            .collect(),
        rewritten: taken.rewritten.iter().map(rewritten).collect(),
    })
}

fn link(roadmap: &Roadmap, id: &str) -> Link {
    Link {
        id: id.to_owned(),
        title: roadmap
            .topics
            .iter()
            .find(|entry| entry.id == id)
            .map_or_else(|| id.to_owned(), |entry| entry.title.clone()),
    }
}

fn rewritten(topic: &Rewritten) -> RewrittenView {
    RewrittenView {
        id: topic.id.clone(),
        title: topic.title.clone(),
        changed: topic
            .changed
            .iter()
            .map(|part| part.label().to_owned())
            .collect(),
        demoted: topic.demoted,
    }
}
