use std::collections::{BTreeSet, HashMap};

use super::violation::Violation;
use crate::roadmap::Roadmap;
use crate::topic::Topic;

pub fn check(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) {
    if roadmap.stages.is_empty() {
        found.push(Violation::EmptyStages);
    }
    if roadmap.topics.is_empty() {
        found.push(Violation::EmptyTopics);
    }
    duplicates(roadmap, found);
    stages(roadmap, found);
    checkpoints(roadmap, found);
    documents(roadmap, topics, found);
    versions(roadmap, topics, found);
    hours(roadmap, topics, found);
}

fn duplicates(roadmap: &Roadmap, found: &mut Vec<Violation>) {
    let mut seen = BTreeSet::new();
    let mut reported = BTreeSet::new();
    for entry in &roadmap.topics {
        if !seen.insert(&entry.id) && reported.insert(&entry.id) {
            found.push(Violation::DuplicateTopicId {
                id: entry.id.clone(),
            });
        }
    }
}

fn stages(roadmap: &Roadmap, found: &mut Vec<Violation>) {
    let known: BTreeSet<u32> = roadmap.stages.iter().map(|stage| stage.n).collect();
    for entry in &roadmap.topics {
        if !known.contains(&entry.stage) {
            found.push(Violation::StageOutOfRange {
                topic: entry.id.clone(),
                stage: entry.stage,
            });
        }
    }
}

fn checkpoints(roadmap: &Roadmap, found: &mut Vec<Violation>) {
    let placement: HashMap<&str, u32> = roadmap
        .topics
        .iter()
        .map(|entry| (entry.id.as_str(), entry.stage))
        .collect();
    for stage in &roadmap.stages {
        match placement.get(stage.checkpoint.as_str()) {
            None => found.push(Violation::UnknownCheckpoint {
                stage: stage.n,
                checkpoint: stage.checkpoint.clone(),
            }),
            Some(&placed) if placed != stage.n => found.push(Violation::CheckpointOutsideStage {
                stage: stage.n,
                checkpoint: stage.checkpoint.clone(),
                found: placed,
            }),
            Some(_) => {}
        }
    }
}

fn hours(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) {
    let entries = roadmap
        .topics
        .iter()
        .map(|entry| (&entry.id, entry.est_hours));
    let documents = topics.iter().map(|topic| (&topic.id, topic.est_hours));
    for (id, hours) in entries.chain(documents) {
        if hours.min > hours.max {
            found.push(Violation::HoursReversed {
                topic: id.clone(),
                min: hours.min,
                max: hours.max,
            });
        }
    }
}

fn documents(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) {
    let generated: BTreeSet<u32> = roadmap
        .stages
        .iter()
        .filter(|stage| stage.generated)
        .map(|stage| stage.n)
        .collect();
    let loaded: BTreeSet<&str> = topics.iter().map(|topic| topic.id.as_str()).collect();
    for entry in &roadmap.topics {
        if generated.contains(&entry.stage) && !loaded.contains(entry.id.as_str()) {
            found.push(Violation::MissingTopicFile {
                topic: entry.id.clone(),
                stage: entry.stage,
                file: entry.file.clone(),
            });
        }
    }
}

fn versions(roadmap: &Roadmap, topics: &[Topic], found: &mut Vec<Violation>) {
    let expected = major(&roadmap.schema);
    for topic in topics {
        let actual = major(&topic.schema);
        if actual != expected {
            found.push(Violation::SchemaMajorMismatch {
                topic: topic.id.clone(),
                roadmap: expected,
                found: actual,
            });
        }
    }
}

fn major(schema: &str) -> u32 {
    schema
        .rsplit('/')
        .next()
        .and_then(|last| last.strip_prefix('v'))
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}
