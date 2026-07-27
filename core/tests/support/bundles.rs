use tolearn_core::roadmap::{Roadmap, parse as roadmap};
use tolearn_core::topic::{Topic, parse as topic};

use super::read;

const REFERENCE: &str = "examples/llm-agents-base";
const REFERENCE_TOPICS: [&str; 7] = [
    "local-runtime",
    "openai-compatible-api",
    "tokens-context-cost",
    "structured-output",
    "model-selection",
    "provider-routing",
    "cp-gateway",
];

const CORPUS: &str = "fixtures/valid/roadmap/corpus-program.yaml";
const CORPUS_TOPICS: [&str; 3] = ["cycle-a", "cycle-b", "stale-knowledge"];

pub fn reference() -> (Roadmap, Vec<Topic>) {
    let map = roadmap(&read(&format!("{REFERENCE}/roadmap.yaml"))).unwrap();
    let topics = REFERENCE_TOPICS
        .iter()
        .map(|name| topic(&read(&format!("{REFERENCE}/topics/{name}.yaml"))).unwrap())
        .collect();
    (map, topics)
}

const CORPUS_ALL: [&str; 5] = [
    "cycle-a",
    "cycle-b",
    "stale-knowledge",
    "offline-edge",
    "question-shapes",
];

pub fn corpus_whole() -> (Roadmap, Vec<Topic>) {
    let map = roadmap(&read(CORPUS)).unwrap();
    let topics = CORPUS_ALL
        .iter()
        .map(|name| topic(&read(&format!("fixtures/valid/topic/{name}.yaml"))).unwrap())
        .collect();
    (map, topics)
}

pub fn corpus() -> (Roadmap, Vec<Topic>) {
    let map = roadmap(&read(CORPUS)).unwrap();
    let topics = CORPUS_TOPICS
        .iter()
        .map(|name| topic(&read(&format!("fixtures/valid/topic/{name}.yaml"))).unwrap())
        .collect();
    (map, topics)
}

pub fn entry(map: &Roadmap, id: &str) -> usize {
    map.topics
        .iter()
        .position(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("`{id}` is not in the roadmap"))
}

pub fn document(topics: &[Topic], id: &str) -> usize {
    topics
        .iter()
        .position(|document| document.id == id)
        .unwrap_or_else(|| panic!("`{id}` is not loaded"))
}
