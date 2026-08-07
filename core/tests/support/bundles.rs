use tolearn_core::progress::{Progress, parse as progress};
use tolearn_core::roadmap::{Roadmap, parse as roadmap};
use tolearn_core::topic::{Question, QuestionType, Topic, parse as topic};

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

pub fn a_question() -> Question {
    Question {
        id: "q1".to_owned(),
        kind: QuestionType::Boundary,
        text: "Чем цикл в графе отличается от заблокированной темы?".to_owned(),
        expected_signals: vec!["Цикл нельзя разорвать порядком".to_owned()],
        red_flags: Vec::new(),
        follow_up: None,
    }
}

pub fn recorded(pairs: &[(&str, &str)]) -> Progress {
    recorded_for("corpus-program", pairs)
}

pub fn recorded_for(program: &str, pairs: &[(&str, &str)]) -> Progress {
    let head = format!("schema: learning-roadmap/progress/v1\nroadmap_id: {program}\ntopics:");
    let mut source = format!("{head}{}\n", if pairs.is_empty() { " {}" } else { "" });
    for (id, status) in pairs {
        source.push_str(&format!(
            "  {id}:\n    status: {status}\n    attempts: []\n    passed_at: null\n    next_review_at: null\n    gaps: []\n"
        ));
    }
    progress(&source).unwrap()
}
