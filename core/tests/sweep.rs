#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "sweep gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::progress::parse as progress;
use tolearn_core::sweep::pool;

use support::{bundles, read};

const TODAY: &str = "2026-07-28";

fn picked(today: &str) -> Vec<(String, Option<String>, bool)> {
    let (roadmap, topics) = bundles::reference();
    let state = progress(&read("fixtures/valid/progress/review-queue.yaml")).unwrap();
    pool(&roadmap, &topics, &state, Date::parse(today).unwrap())
        .into_iter()
        .map(|item| (item.topic, item.due, item.overdue))
        .collect()
}

#[test]
fn в_прогон_идут_только_пройденные_темы() {
    let (roadmap, topics) = bundles::reference();
    let state = progress(&read("fixtures/valid/progress/review-queue.yaml")).unwrap();
    let ready = pool(&roadmap, &topics, &state, Date::parse(TODAY).unwrap());

    let ids: Vec<&str> = ready.iter().map(|item| item.topic.as_str()).collect();
    assert!(!ids.contains(&"prompt-basics"));
    assert!(ready.iter().all(|item| {
        topics
            .iter()
            .any(|topic| topic.id == item.topic && !topic.questions.is_empty())
    }));
}

#[test]
fn просроченные_повторения_идут_первыми() {
    let order: Vec<String> = picked(TODAY)
        .into_iter()
        .map(|(topic, _, _)| topic)
        .collect();

    assert_eq!(
        order.first().map(String::as_str),
        Some("tokens-context-cost")
    );
    assert_eq!(order.get(1).map(String::as_str), Some("local-runtime"));
    assert_eq!(
        order.get(2).map(String::as_str),
        Some("openai-compatible-api")
    );
}

#[test]
fn не_подошедшее_по_дате_остаётся_позади_подошедшего() {
    let ready = picked(TODAY);
    let waiting = ready
        .iter()
        .position(|(topic, _, _)| topic == "structured-output")
        .unwrap();
    let due = ready
        .iter()
        .position(|(topic, _, _)| topic == "openai-compatible-api")
        .unwrap();

    assert!(due < waiting);
}

#[test]
fn просрочка_помечается_флагом() {
    let ready = picked(TODAY);

    let overdue: Vec<&String> = ready
        .iter()
        .filter(|(_, _, overdue)| *overdue)
        .map(|(topic, _, _)| topic)
        .collect();
    assert_eq!(overdue, vec!["tokens-context-cost", "local-runtime"]);
    assert!(ready.iter().any(
        |(topic, due, _)| topic == "structured-output" && due.as_deref() == Some("2026-10-24")
    ));
}
