#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "writer gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::progress::{
    Document, DocumentError, Format, Progress, Source, Status, TopicState, Verdict, parse,
};
use tolearn_core::status::manual;
use tolearn_core::topic::Topic;

use support::{bundles, read};

const HISTORY: &str = "fixtures/valid/progress/attempts-history.yaml";
const CORPUS: &str = "fixtures/valid/progress/corpus-program.yaml";
const AT: &str = "2026-07-27T10:00:00Z";

const EVERY_STATUS: [Status; 8] = [
    Status::Todo,
    Status::InProgress,
    Status::ExamPending,
    Status::Passed,
    Status::PassedOut,
    Status::StalePassed,
    Status::Blocked,
    Status::Failed,
];

fn document(source: &str) -> Document {
    Document::read(&read(source), Format::Yaml).unwrap()
}

fn topic(id: &str) -> Topic {
    let (_, topics) = bundles::corpus_whole();
    topics[bundles::document(&topics, id)].clone()
}

fn state<'a>(progress: &'a Progress, id: &str) -> &'a TopicState {
    progress
        .state(id)
        .unwrap_or_else(|| panic!("`{id}` is not in the progress"))
}

fn day(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|| panic!("`{text}` is no date"))
}

#[test]
fn any_status_is_set_by_hand_without_an_exam() {
    for status in EVERY_STATUS {
        let mut file = document(CORPUS);
        let mark = manual(status, AT, day("2026-07-27"), &topic("stale-knowledge"));

        file.mark("stale-knowledge", &mark).unwrap();

        assert_eq!(
            state(file.progress(), "stale-knowledge").status,
            status,
            "{status:?}"
        );
    }
}

#[test]
fn the_mark_is_written_as_an_attempt_made_by_hand() {
    let mut file = document(CORPUS);
    let mark = manual(
        Status::Passed,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );

    file.mark("stale-knowledge", &mark).unwrap();

    let written = state(file.progress(), "stale-knowledge")
        .attempts
        .last()
        .unwrap()
        .clone();
    assert_eq!(written.source, Source::Manual);
    assert_eq!(written.at, AT);
    assert_eq!(written.raw, "");
    assert!(written.per_question.is_empty());
}

#[test]
fn the_verdict_of_a_mark_follows_the_status_it_sets() {
    fn expected(status: Status) -> Verdict {
        match status {
            Status::Passed | Status::PassedOut | Status::StalePassed => Verdict::Pass,
            Status::Failed => Verdict::Fail,
            Status::Blocked => Verdict::Blocked,
            Status::Todo | Status::InProgress | Status::ExamPending => Verdict::Partial,
        }
    }

    for status in EVERY_STATUS {
        let mut file = document(CORPUS);
        let mark = manual(status, AT, day("2026-07-27"), &topic("stale-knowledge"));

        file.mark("stale-knowledge", &mark).unwrap();

        let written = state(file.progress(), "stale-knowledge")
            .attempts
            .last()
            .unwrap()
            .clone();
        assert_eq!(written.verdict, expected(status), "{status:?}");
    }
}

#[test]
fn a_note_of_the_human_is_kept_with_the_mark() {
    let mut file = document(CORPUS);
    let mut mark = manual(
        Status::Passed,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );
    mark.note = Some("Прошёл эту тему на работе.\n".to_owned());

    file.mark("stale-knowledge", &mark).unwrap();

    let written = state(file.progress(), "stale-knowledge")
        .attempts
        .last()
        .unwrap()
        .clone();
    assert_eq!(written.notes, ["Прошёл эту тему на работе.\n"]);
}

#[test]
fn a_reset_by_hand_leaves_the_history_of_attempts_alone() {
    let mut file = document(CORPUS);
    let before = state(file.progress(), "stale-knowledge").attempts.clone();
    let mark = manual(
        Status::Todo,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );

    file.mark("stale-knowledge", &mark).unwrap();

    let after = state(file.progress(), "stale-knowledge").attempts.clone();
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(after[..before.len()], before[..]);
}

#[test]
fn a_pass_by_hand_dates_itself_and_asks_for_the_next_review() {
    let mut file = document(CORPUS);
    let mark = manual(
        Status::Passed,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );

    file.mark("stale-knowledge", &mark).unwrap();

    let written = state(file.progress(), "stale-knowledge");
    assert_eq!(written.passed_at.as_deref(), Some("2026-07-27"));
    assert_eq!(written.next_review_at.as_deref(), Some("2026-08-26"));
}

#[test]
fn a_pass_by_hand_of_a_topic_kept_by_use_counts_the_review_from_the_verification() {
    let mut file = document(CORPUS);
    let mark = manual(
        Status::Passed,
        AT,
        day("2026-07-27"),
        &topic("offline-edge"),
    );

    file.mark("offline-edge", &mark).unwrap();

    let written = state(file.progress(), "offline-edge");
    assert_eq!(written.next_review_at.as_deref(), Some("2026-10-18"));
}

#[test]
fn a_reset_by_hand_clears_the_dates_of_the_pass() {
    let mut file = document(CORPUS);
    let mark = manual(
        Status::Todo,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );

    file.mark("stale-knowledge", &mark).unwrap();

    let written = state(file.progress(), "stale-knowledge");
    assert_eq!(written.passed_at, None);
    assert_eq!(written.next_review_at, None);
}

#[test]
fn the_marked_document_is_still_a_progress_file() {
    let mut file = document(CORPUS);
    let mark = manual(
        Status::Passed,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );

    file.mark("stale-knowledge", &mark).unwrap();

    assert_eq!(&parse(file.text()).unwrap(), file.progress());
}

#[test]
fn a_mark_on_a_topic_the_file_does_not_know_is_refused() {
    let mut file = document(CORPUS);
    let mark = manual(Status::Passed, AT, day("2026-07-27"), &topic("cycle-a"));

    let error = file.mark("no-such-topic", &mark).unwrap_err();

    assert!(
        matches!(&error, DocumentError::UnknownTopic(id) if id == "no-such-topic"),
        "{error}"
    );
}

#[test]
fn the_statistics_tell_a_mark_by_hand_from_a_passed_exam() {
    let file = document(HISTORY);
    let passed = state(file.progress(), "local-runtime");

    assert_eq!(passed.marked_by(), Some(Source::Exam));

    let mut file = document(HISTORY);
    let mark = manual(
        Status::Passed,
        AT,
        day("2026-07-27"),
        &topic("stale-knowledge"),
    );
    file.mark("local-runtime", &mark).unwrap();

    assert_eq!(
        state(file.progress(), "local-runtime").marked_by(),
        Some(Source::Manual)
    );
}

#[test]
fn a_topic_nobody_touched_was_marked_by_nobody() {
    let file = document(CORPUS);

    assert_eq!(state(file.progress(), "cycle-b").marked_by(), None);
}

#[test]
fn a_mark_that_is_not_a_pass_gets_no_dates_even_for_a_topic_kept_by_use() {
    let mut file = document(CORPUS);
    let mark = manual(
        Status::Failed,
        AT,
        day("2026-07-27"),
        &topic("offline-edge"),
    );

    file.mark("offline-edge", &mark).unwrap();

    let written = state(file.progress(), "offline-edge");
    assert_eq!(written.passed_at, None);
    assert_eq!(written.next_review_at, None);
}

#[test]
fn a_mark_by_hand_carries_the_dates_the_caller_put_in_it() {
    let mut file = document(CORPUS);
    let mut mark = manual(Status::Passed, AT, day("2026-07-27"), &topic("cycle-a"));
    mark.passed_at = Some("2026-01-01".to_owned());
    mark.next_review_at = None;

    file.mark("cycle-a", &mark).unwrap();

    let written = state(file.progress(), "cycle-a");
    assert_eq!(written.passed_at.as_deref(), Some("2026-01-01"));
    assert_eq!(written.next_review_at, None);
}
