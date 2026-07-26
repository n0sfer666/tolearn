#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::topic::{
    Check, Liveness, MaterialTier, MaterialType, PracticeKind, PracticeTier, QuestionType, Topic,
    parse,
};

use support::read;

const REFERENCE: &str = "examples/llm-agents-base/topics/local-runtime.yaml";
const PAYWALL: &str = "fixtures/valid/topic/offline-edge.yaml";

fn reference() -> Topic {
    parse(&read(REFERENCE)).unwrap()
}

#[test]
fn constraints_and_acceptance_are_separate_collections() {
    let practice = reference().practice;

    let constraints: Vec<&str> = practice.constraints.iter().map(id).collect();
    let acceptance: Vec<&str> = practice.acceptance.iter().map(id).collect();

    assert_eq!(constraints, ["c1", "c2", "c3"]);
    assert_eq!(acceptance, ["a1", "a2", "a3", "a4", "a5", "a6"]);
    assert!(
        practice
            .constraints
            .iter()
            .all(|check| !practice.acceptance.contains(check)),
        "a check leaked from one collection into the other"
    );
    assert!(
        practice.acceptance[0]
            .claim
            .starts_with("Все восемь артефактов"),
        "{}",
        practice.acceptance[0].claim
    );
    assert!(
        practice.constraints[0]
            .claim
            .starts_with("Контекст в Ollama"),
        "{}",
        practice.constraints[0].claim
    );
}

fn id(check: &Check) -> &str {
    &check.id
}

#[test]
fn block_scalars_keep_their_line_breaks() {
    let topic = reference();

    let task = topic.practice.task;
    assert!(task.lines().count() > 20, "{}", task.lines().count());
    assert!(task.ends_with('\n'), "{task:?}");
    assert!(
        task.contains("\n\n"),
        "the blank line inside the task is gone"
    );
    assert!(
        task.contains("\n1. Запиши в `predict.md`"),
        "the numbered list lost the margin the block indicator gave it"
    );
    assert!(
        task.contains("\n   весит файл модели"),
        "the continuation line lost the indentation it has relative to its item"
    );

    assert_eq!(topic.outcomes[0].lines().count(), 3);
    assert!(topic.outcomes[0].ends_with(".\n"));
    assert_eq!(topic.exam.focus.lines().count(), 6);

    let check = &topic.practice.constraints[0].check;
    assert_eq!(check.lines().count(), 1);
    assert!(check.ends_with('\n'), "{check:?}");
}

#[test]
fn a_missing_optional_value_is_none_not_an_empty_string() {
    let topic = reference();

    let first = &topic.materials[0];
    assert_eq!(first.published, None);
    assert_eq!(first.covers_version.as_deref(), Some("v0.32.4"));
    assert_eq!(first.delta, None);

    let dated = &topic.materials[3];
    assert_eq!(dated.published.as_deref(), Some("2024-10"));
    assert_eq!(dated.covers_version, None);
    assert!(dated.stale);
    assert!(dated.delta.is_some());

    assert_eq!(topic.practice.starting_point, None);
    assert_eq!(topic.practice.fallback, None);
    assert!(topic.questions[0].follow_up.is_some());
}

#[test]
fn every_closed_vocabulary_of_the_reference_is_read() {
    let topic = reference();

    assert_eq!(topic.materials[0].kind, MaterialType::Docs);
    assert_eq!(topic.materials[0].tier, MaterialTier::T1);
    assert_eq!(topic.materials[0].lang, "en");
    assert_eq!(topic.materials[0].liveness, Liveness::Ok);
    assert_eq!(topic.materials[3].kind, MaterialType::Article);
    assert_eq!(topic.materials[3].tier, MaterialTier::T2);

    assert_eq!(topic.practice.kind, PracticeKind::Ops);
    assert_eq!(topic.practice.tier, PracticeTier::P1);
    assert_eq!(topic.practice.time_box_min, 75);
    assert!(!topic.practice.smoke_checked);

    let types: Vec<QuestionType> = topic.questions.iter().map(|q| q.kind).collect();
    assert_eq!(
        types,
        [
            QuestionType::Diagnose,
            QuestionType::Predict,
            QuestionType::Boundary,
            QuestionType::Misconception,
        ]
    );
    assert_eq!(topic.questions[0].expected_signals.len(), 4);
    assert_eq!(topic.questions[0].red_flags.len(), 3);
}

#[test]
fn a_material_behind_a_paywall_keeps_its_liveness() {
    let topic = parse(&read(PAYWALL)).unwrap();

    let kinds: Vec<Liveness> = topic
        .materials
        .iter()
        .map(|material| material.liveness)
        .collect();

    assert!(
        kinds.contains(&Liveness::Paywall),
        "the fixture stopped covering paywalled materials: {kinds:?}"
    );
}
