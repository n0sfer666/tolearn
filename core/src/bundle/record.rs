use std::collections::BTreeSet;

use super::violation::Violation;
use crate::progress::Progress;
use crate::roadmap::Roadmap;

pub fn check(roadmap: &Roadmap, progress: &Progress, found: &mut Vec<Violation>) {
    if progress.roadmap_id != roadmap.id {
        found.push(Violation::ProgressForAnotherProgram {
            program: roadmap.id.clone(),
            found: progress.roadmap_id.clone(),
        });
    }

    let recorded: BTreeSet<&str> = progress.topics.iter().map(|(id, _)| id.as_str()).collect();
    for entry in &roadmap.topics {
        if !recorded.contains(entry.id.as_str()) {
            found.push(Violation::UntrackedTopic {
                topic: entry.id.clone(),
            });
        }
    }

    let declared: BTreeSet<&str> = roadmap
        .topics
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    for (id, _) in &progress.topics {
        if !declared.contains(id.as_str()) {
            found.push(Violation::StrayProgressTopic { topic: id.clone() });
        }
    }
}
