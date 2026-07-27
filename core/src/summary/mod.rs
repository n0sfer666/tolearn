mod types;

pub use types::{Stage, Summary, Tally};

use crate::roadmap::{Priority, Roadmap};
use crate::status::Statuses;
use crate::topic::Topic;

pub fn summarize(roadmap: &Roadmap, topics: &[Topic], statuses: &Statuses) -> Summary {
    let mut stages: Vec<Stage> = roadmap
        .stages
        .iter()
        .map(|stage| Stage {
            n: stage.n,
            title: stage.title.clone(),
            tally: Tally::default(),
        })
        .collect();
    let mut program = Tally::default();

    for entry in &roadmap.topics {
        if entry.priority == Priority::Optional {
            continue;
        }
        if !topics.iter().any(|topic| topic.id == entry.id) {
            continue;
        }
        let Some(status) = statuses.get(&entry.id) else {
            continue;
        };
        program.add(entry.est_hours, status);
        if let Some(stage) = stages.iter_mut().find(|stage| stage.n == entry.stage) {
            stage.tally.add(entry.est_hours, status);
        }
    }

    Summary { program, stages }
}
