mod prune;
mod types;

pub use prune::prune;
pub use types::{Diff, Kept, Rewritten, Snapshot};

use std::collections::BTreeSet;

use crate::merge::changed;
use crate::progress::{Progress, Status};
use crate::topic::Topic;

pub fn diff(before: Snapshot<'_>, after: Snapshot<'_>, progress: &Progress) -> Diff {
    let was: BTreeSet<&str> = ids(before);
    let is: BTreeSet<&str> = ids(after);

    Diff {
        added: is.difference(&was).map(|id| (*id).to_owned()).collect(),
        removed: was.difference(&is).map(|id| (*id).to_owned()).collect(),
        rewritten: rewritten(before, after, progress),
    }
}

fn ids<'a>(snapshot: Snapshot<'a>) -> BTreeSet<&'a str> {
    snapshot
        .roadmap
        .topics
        .iter()
        .map(|entry| entry.id.as_str())
        .collect()
}

fn rewritten(before: Snapshot<'_>, after: Snapshot<'_>, progress: &Progress) -> Vec<Rewritten> {
    after
        .roadmap
        .topics
        .iter()
        .filter_map(|entry| {
            let (was, is) = (
                document(before.topics, &entry.id)?,
                document(after.topics, &entry.id)?,
            );
            let parts = changed(was, is);
            if parts.is_empty() {
                return None;
            }
            Some(Rewritten {
                id: entry.id.clone(),
                title: entry.title.clone(),
                changed: parts,
                demoted: demoted(progress, &entry.id),
            })
        })
        .collect()
}

fn document<'a>(topics: &'a [Topic], id: &str) -> Option<&'a Topic> {
    topics.iter().find(|topic| topic.id == id)
}

fn demoted(progress: &Progress, id: &str) -> bool {
    progress
        .state(id)
        .is_some_and(|state| state.status == Status::StalePassed)
}
