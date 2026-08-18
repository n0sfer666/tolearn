#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "queue gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::progress::parse as progress;
use tolearn_core::queue::{due, repeat};
use tolearn_core::topic::parse as topic;

use support::{bundles, read};

const TODAY: &str = "2026-07-28";

fn queued(today: &str) -> Vec<(String, bool, String)> {
    let (_, topics) = bundles::reference();
    let state = progress(&read("fixtures/valid/progress/review-queue.yaml")).unwrap();
    due(&topics, &state, Date::parse(today).unwrap())
        .into_iter()
        .map(|item| (item.topic, item.overdue, item.due))
        .collect()
}

#[test]
fn очередь_собирается_по_дате_повторения() {
    let queue = queued(TODAY);

    assert_eq!(
        queue,
        vec![
            (
                "tokens-context-cost".to_owned(),
                true,
                "2026-07-09".to_owned()
            ),
            ("local-runtime".to_owned(), true, "2026-07-20".to_owned()),
            (
                "openai-compatible-api".to_owned(),
                false,
                "2026-07-28".to_owned()
            ),
        ]
    );
}

#[test]
fn просроченное_отделено_от_сегодняшнего() {
    let queue = queued(TODAY);
    let overdue: Vec<&String> = queue
        .iter()
        .filter(|(_, overdue, _)| *overdue)
        .map(|(topic, _, _)| topic)
        .collect();

    assert_eq!(overdue, vec!["tokens-context-cost", "local-runtime"]);
    assert!(queue.iter().any(|(_, overdue, _)| !overdue));
}

#[test]
fn будущее_повторение_в_очередь_не_попадает() {
    let queue = queued(TODAY);

    assert!(
        !queue
            .iter()
            .any(|(topic, _, _)| topic == "structured-output"),
        "{queue:?}"
    );
}

#[test]
fn тема_без_даты_повторения_очереди_не_касается() {
    let queue = queued("2027-01-01");

    assert!(
        !queue
            .iter()
            .any(|(topic, _, _)| topic == "model-selection" || topic == "provider-routing"),
        "{queue:?}"
    );
}

#[test]
fn очередь_знает_название_темы() {
    let (_, topics) = bundles::reference();
    let state = progress(&read("fixtures/valid/progress/review-queue.yaml")).unwrap();

    let queue = due(&topics, &state, Date::parse(TODAY).unwrap());

    let first = queue.first().unwrap();
    let named = topics.iter().find(|topic| topic.id == first.topic).unwrap();
    assert_eq!(first.title, named.title);
}

#[test]
fn повторение_двигает_дату_на_срок_ревалидации() {
    let subject = topic(&read("examples/llm-agents-base/topics/local-runtime.yaml")).unwrap();

    let moved = repeat(&subject, Date::parse(TODAY).unwrap()).unwrap();

    assert_eq!(moved.to_string(), "2026-10-26");
}

#[test]
fn тема_без_ревизий_даты_не_получает() {
    let mut subject = topic(&read("examples/llm-agents-base/topics/local-runtime.yaml")).unwrap();
    subject.retention = tolearn_core::topic::Retention::None;

    assert_eq!(repeat(&subject, Date::parse(TODAY).unwrap()), None);
}

#[test]
fn перенос_даты_не_трогает_ни_статус_ни_попытки() {
    let source = read("fixtures/valid/progress/review-queue.yaml");
    let mut document =
        tolearn_core::progress::Document::read(&source, tolearn_core::progress::Format::Yaml)
            .unwrap();
    let before = document
        .progress()
        .state("tokens-context-cost")
        .unwrap()
        .clone();

    document
        .reschedule("tokens-context-cost", "2026-10-26")
        .unwrap();

    let after = document.progress().state("tokens-context-cost").unwrap();
    assert_eq!(after.next_review_at.as_deref(), Some("2026-10-26"));
    assert_eq!(after.status, before.status);
    assert_eq!(after.attempts, before.attempts);
    assert_eq!(after.passed_at, before.passed_at);
    assert_eq!(after.gaps, before.gaps);
}
