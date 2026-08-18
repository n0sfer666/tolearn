use tolearn_core::merge::merge;
use tolearn_core::progress::{Document, Format};
use tolearn_core::roadmap::parse as roadmap;
use tolearn_core::topic::parse as topic;

use crate::repo::read;
use crate::schema::{validator, yaml::load};

const PROGRESS: &str = "fixtures/valid/progress/corpus-program.yaml";
const ROADMAP: &str = "fixtures/valid/roadmap/corpus-program.yaml";
const TOPICS: [&str; 5] = [
    "cycle-a",
    "cycle-b",
    "stale-knowledge",
    "offline-edge",
    "question-shapes",
];

#[test]
fn the_progress_a_regeneration_leaves_behind_still_validates() {
    let mut map = roadmap(&read(ROADMAP)).unwrap();
    let before: Vec<_> = TOPICS
        .iter()
        .map(|name| topic(&read(&format!("fixtures/valid/topic/{name}.yaml"))).unwrap())
        .collect();

    let mut after = before.clone();
    after[0].exam.focus = "Переписанный экзамен".to_owned();
    let mut fresh = after[0].clone();
    fresh.id = "fresh-topic".to_owned();
    let mut entry = map.topics[0].clone();
    entry.id = "fresh-topic".to_owned();
    map.topics.push(entry);
    map.topics.retain(|entry| entry.id != "question-shapes");
    after.push(fresh);

    let mut document = Document::read(&read(PROGRESS), Format::Yaml).unwrap();
    let report = merge(&mut document, &map, &before, &after).unwrap();
    assert_eq!(report.added, ["fresh-topic"]);
    assert_eq!(report.orphaned, ["question-shapes"]);

    let validator = validator("progress");
    let instance = load(document.text(), "the merged progress");
    let complaints: Vec<String> = validator
        .iter_errors(&instance)
        .map(|error| format!("  {}: {error}", error.instance_path()))
        .collect();
    assert!(
        complaints.is_empty(),
        "a merged progress is expected to stay valid:\n{}",
        complaints.join("\n")
    );
}
