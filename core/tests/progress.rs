#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::progress::{NextAction, Outcome, Source, Status, Verdict, parse};

use support::{every_fixture_parses, read};

fn history() -> tolearn_core::progress::Progress {
    parse(&read("fixtures/valid/progress/attempts-history.yaml")).unwrap()
}

#[test]
fn every_valid_progress_fixture_parses() {
    every_fixture_parses("fixtures/valid/progress", parse, 3);
}

#[test]
fn the_reference_progress_reads_every_topic_of_the_first_stage() {
    let progress = parse(&read("examples/llm-agents-base/progress.yaml")).unwrap();

    assert_eq!(progress.schema, "learning-roadmap/progress/v1");
    assert_eq!(progress.roadmap_id, "llm-agents-base");
    assert_eq!(progress.topics.len(), 7);
    assert!(
        progress
            .topics
            .iter()
            .all(|(_, state)| state.status == Status::Todo && state.attempts.is_empty())
    );
}

#[test]
fn topics_keep_the_order_they_have_in_the_file() {
    let progress = parse(&read("examples/llm-agents-base/progress.yaml")).unwrap();
    let names: Vec<&str> = progress.topics.iter().map(|(id, _)| id.as_str()).collect();

    assert_eq!(names.first().copied(), Some("local-runtime"));
    assert_eq!(names.last().copied(), Some("cp-gateway"));
}

#[test]
fn a_passed_topic_carries_its_dates_and_every_attempt() {
    let progress = history();
    let state = progress.state("local-runtime").unwrap();

    assert_eq!(state.status, Status::Passed);
    assert_eq!(state.passed_at.as_deref(), Some("2026-07-23"));
    assert_eq!(state.next_review_at.as_deref(), Some("2026-10-24"));
    assert_eq!(state.attempts.len(), 2);
}

#[test]
fn an_attempt_carries_the_whole_verdict_in_the_shape_of_the_protocol() {
    let progress = history();
    let state = progress.state("local-runtime").unwrap();
    let attempt = &state.attempts[0];

    assert_eq!(attempt.at, "2026-07-20T11:05:00Z");
    assert_eq!(attempt.source, Source::Exam);
    assert_eq!(attempt.verdict, Verdict::Partial);
    assert_eq!(attempt.model.as_deref(), Some("claude-opus-5"));
    assert!(!attempt.hinted);
    assert!(!attempt.practice_accepted);
    assert_eq!(attempt.failed_checks, ["a2"]);
    assert_eq!(attempt.gaps.len(), 1);
    assert!(attempt.calibration.is_some());
    assert_eq!(attempt.next_action, Some(NextAction::RetryFailed));
    assert_eq!(attempt.retry_after_days, Some(2));
}

#[test]
fn every_answer_of_an_attempt_is_read_apart() {
    let progress = history();
    let attempt = &progress.state("local-runtime").unwrap().attempts[0];
    let outcomes: Vec<Outcome> = attempt
        .per_question
        .iter()
        .map(|answer| answer.outcome)
        .collect();

    assert_eq!(outcomes, [Outcome::Ok, Outcome::Partial, Outcome::Miss]);
    assert_eq!(attempt.per_question[0].id, "q1");
    assert!(attempt.per_question[2].quote.is_none());
    assert_eq!(attempt.per_question[2].missed.len(), 1);
    assert!(!attempt.per_question[1].signal_extension);
    assert!(progress.state("local-runtime").unwrap().attempts[1].per_question[0].signal_extension);
}

#[test]
fn the_raw_answer_is_kept_word_for_word() {
    let source = read("fixtures/valid/progress/attempts-history.yaml");
    let progress = parse(&source).unwrap();
    let raw = &progress.state("local-runtime").unwrap().attempts[0].raw;

    assert!(raw.starts_with("Модель занимает столько, сколько весит файл"));
    assert!(raw.ends_with("точнее не скажу.\n"), "{raw:?}");
    assert!(raw.lines().count() == 3, "{raw:?}");
}

#[test]
fn the_fields_a_verdict_may_leave_out_are_optional() {
    let progress = history();
    let attempt = &progress.state("tokens-context-cost").unwrap().attempts[0];

    assert_eq!(attempt.source, Source::Manual);
    assert!(attempt.model.is_none());
    assert!(attempt.failed_checks.is_empty());
    assert!(attempt.per_question.is_empty());
    assert!(attempt.calibration.is_none());
    assert!(attempt.retry_after_days.is_none());
    assert_eq!(attempt.raw, "");
}

#[test]
fn a_topic_that_was_never_touched_carries_no_dates() {
    let progress = history();
    let state = progress.state("openai-compatible-api").unwrap();

    assert_eq!(state.status, Status::InProgress);
    assert!(state.attempts.is_empty());
    assert!(state.passed_at.is_none());
    assert!(state.next_review_at.is_none());
    assert!(state.gaps.is_empty());
}

#[test]
fn every_status_of_the_schema_is_read() {
    let history = history();
    let corpus = parse(&read("fixtures/valid/progress/corpus-program.yaml")).unwrap();
    let seen: Vec<Status> = history
        .topics
        .iter()
        .chain(corpus.topics.iter())
        .map(|(_, state)| state.status)
        .collect();

    for status in [
        Status::Todo,
        Status::InProgress,
        Status::ExamPending,
        Status::Passed,
        Status::PassedOut,
        Status::StalePassed,
        Status::Blocked,
        Status::Failed,
    ] {
        assert!(seen.contains(&status), "{status:?} is in no fixture");
    }
}
