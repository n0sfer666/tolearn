#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "protocol gate: a panic here is the report"
)]

mod support;

use tolearn_core::Date;
use tolearn_core::progress::{Attempt, Source, Status, TopicState, Verdict as Grade};
use tolearn_core::protocol::{apply, counted, failing_streak};
use tolearn_core::topic::Topic;
use tolearn_core::verdict::{Verdict, parse};

use support::bundles;

const AT: &str = "2026-07-27T18:40:00+03:00";
const MISS: &str = r#"[{"id": "q1", "result": "miss"}]"#;

fn topic(id: &str) -> Topic {
    let (_, topics) = bundles::corpus_whole();
    topics[bundles::document(&topics, id)].clone()
}

fn graded(topic: &Topic, result: &str, answers: &str) -> Verdict {
    let text = format!(
        "{{\"topic_id\": \"{}\", \"date\": \"2026-07-27\", \"verdict\": \"{result}\",
          \"per_question\": {answers}, \"gaps\": [\"порядок обхода\"],
          \"retry_after_days\": 3}}",
        topic.id
    );
    parse(&text, topic).unwrap()
}

fn given(topic: &Topic, result: &str) -> Verdict {
    graded(topic, result, MISS)
}

fn today() -> Date {
    Date::parse("2026-07-27").unwrap()
}

fn state(results: &[&str]) -> TopicState {
    let subject = topic("stale-knowledge");
    let mut state = None;
    for result in results {
        let applied = apply(
            &given(&subject, result),
            &subject,
            state.as_ref(),
            AT,
            today(),
        );
        state = Some(applied.state);
    }
    state.unwrap_or_else(|| panic!("at least one verdict is expected"))
}

#[test]
fn a_pass_marks_the_topic_passed_on_the_day_of_the_verdict() {
    let subject = topic("stale-knowledge");
    let later = Date::parse("2026-07-30").unwrap();

    let applied = apply(&given(&subject, "pass"), &subject, None, AT, later);

    assert_eq!(applied.state.status, Status::Passed);
    assert_eq!(applied.state.passed_at.as_deref(), Some("2026-07-27"));
}

#[test]
fn a_pass_schedules_the_review_by_the_retention_of_the_topic() {
    let subject = topic("stale-knowledge");

    let applied = apply(&given(&subject, "pass"), &subject, None, AT, today());

    assert_eq!(applied.state.next_review_at.as_deref(), Some("2026-08-26"));
}

#[test]
fn a_topic_kept_by_use_is_reviewed_from_the_day_it_was_verified() {
    let subject = topic("cycle-a");

    let applied = apply(&given(&subject, "pass"), &subject, None, AT, today());

    assert_eq!(applied.state.next_review_at.as_deref(), Some("2028-07-26"));
}

#[test]
fn a_topic_that_does_not_come_back_gets_no_review_date() {
    let subject = topic("question-shapes");

    let applied = apply(&given(&subject, "pass"), &subject, None, AT, today());

    assert_eq!(applied.state.next_review_at, None);
}

#[test]
fn a_partial_leaves_the_topic_in_progress_and_queues_what_was_not_answered() {
    let subject = topic("question-shapes");
    let answers = r#"[{"id": "q1", "result": "ok"}, {"id": "q2", "result": "partial"},
                      {"id": "q3", "result": "miss"}]"#;

    let applied = apply(
        &graded(&subject, "partial", answers),
        &subject,
        None,
        AT,
        today(),
    );

    assert_eq!(applied.state.status, Status::InProgress);
    assert_eq!(applied.retry, ["q2", "q3"]);
}

#[test]
fn a_fail_marks_the_topic_failed_and_keeps_the_gaps_of_the_verdict() {
    let subject = topic("stale-knowledge");

    let applied = apply(&given(&subject, "fail"), &subject, None, AT, today());

    assert_eq!(applied.state.status, Status::Failed);
    assert_eq!(applied.state.gaps, ["порядок обхода"]);
}

#[test]
fn a_blocked_exam_leaves_the_topic_in_progress() {
    let subject = topic("stale-knowledge");

    let applied = apply(&given(&subject, "blocked"), &subject, None, AT, today());

    assert_eq!(applied.state.status, Status::InProgress);
}

#[test]
fn every_verdict_is_recorded_as_an_exam_attempt_with_its_own_text() {
    let subject = topic("stale-knowledge");
    let verdict = given(&subject, "pass");

    let applied = apply(&verdict, &subject, None, AT, today());

    let [attempt] = applied.state.attempts.as_slice() else {
        panic!("one attempt is expected");
    };
    assert_eq!(attempt.at, AT);
    assert_eq!(attempt.source, Source::Exam);
    assert_eq!(attempt.raw, verdict.raw);
    assert_eq!(attempt.retry_after_days, Some(3));
}

#[test]
fn a_blocked_attempt_is_kept_in_the_history_but_out_of_the_statistics() {
    let state = state(&["pass", "blocked"]);

    assert_eq!(state.attempts.len(), 2);
    assert_eq!(counted(&state.attempts).len(), 1);
}

#[test]
fn three_failures_in_a_row_suggest_splitting_the_topic() {
    let subject = topic("stale-knowledge");
    let twice = state(&["fail", "fail"]);

    let applied = apply(
        &given(&subject, "fail"),
        &subject,
        Some(&twice),
        AT,
        today(),
    );

    assert!(applied.split_suggested);
}

#[test]
fn two_failures_are_not_enough_for_a_split() {
    let subject = topic("stale-knowledge");
    let once = state(&["fail"]);

    let applied = apply(&given(&subject, "fail"), &subject, Some(&once), AT, today());

    assert!(!applied.split_suggested);
}

#[test]
fn a_blocked_exam_between_failures_does_not_break_the_run() {
    let state = state(&["fail", "blocked", "fail", "fail"]);

    assert!(failing_streak(&state.attempts) >= 3);
}

#[test]
fn any_other_verdict_breaks_the_run() {
    let state = state(&["fail", "fail", "partial", "fail"]);

    assert_eq!(failing_streak(&state.attempts), 1);
}

#[test]
fn a_verdict_without_a_date_of_its_own_is_dated_by_the_day_it_is_applied() {
    let subject = topic("stale-knowledge");
    let text = format!(
        "{{\"topic_id\": \"{}\", \"verdict\": \"pass\", \"per_question\": {MISS}}}",
        subject.id
    );
    let verdict = parse(&text, &subject).unwrap();
    let later = Date::parse("2026-07-30").unwrap();

    let applied = apply(&verdict, &subject, None, AT, later);

    assert_eq!(applied.state.passed_at.as_deref(), Some("2026-07-30"));
}

#[test]
fn a_verdict_that_is_no_pass_leaves_the_earlier_pass_where_it_was() {
    let subject = topic("stale-knowledge");
    let passed = state(&["pass"]);

    let applied = apply(
        &given(&subject, "fail"),
        &subject,
        Some(&passed),
        AT,
        today(),
    );

    assert_eq!(applied.state.passed_at.as_deref(), Some("2026-07-27"));
    assert_eq!(applied.state.next_review_at.as_deref(), Some("2026-08-26"));
}

#[test]
fn a_pass_clears_the_gaps_a_failure_left() {
    let state = state(&["fail", "pass"]);

    assert!(state.gaps.is_empty());
}

#[test]
fn a_manual_failure_counts_towards_the_run_as_well() {
    let subject = topic("stale-knowledge");
    let mut twice = state(&["fail"]);
    twice.attempts.push(manual_failure());

    let applied = apply(
        &given(&subject, "fail"),
        &subject,
        Some(&twice),
        AT,
        today(),
    );

    assert!(applied.split_suggested);
}

fn manual_failure() -> Attempt {
    Attempt {
        at: AT.to_owned(),
        source: Source::Manual,
        verdict: Grade::Fail,
        model: None,
        hinted: false,
        practice_accepted: false,
        failed_checks: Vec::new(),
        per_question: Vec::new(),
        gaps: Vec::new(),
        calibration: None,
        notes: Vec::new(),
        next_action: None,
        retry_after_days: None,
        raw: String::new(),
    }
}
