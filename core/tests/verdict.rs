#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "verdict gate: a panic here is the report"
)]

mod support;

use tolearn_core::progress::{NextAction, Outcome, Verdict as Result};
use tolearn_core::topic::Topic;
use tolearn_core::verdict::{Verdict, VerdictError, parse};

use support::bundles;

const WHOLE: &str = r#"{
  "topic_id": "question-shapes",
  "date": "2026-07-27",
  "model": "opus",
  "verdict": "partial",
  "hinted": true,
  "practice_accepted": false,
  "failed_checks": ["a1"],
  "per_question": [
    {"id": "q1", "result": "miss", "quote": "ну, зависит",
     "missed": ["порядок обхода"], "signal_extension": false}
  ],
  "gaps": ["не держит порядок"],
  "calibration": "сказал про обход",
  "notes": ["просил подсказку"],
  "next_action": "retry_failed",
  "retry_after_days": 3
}"#;

fn topic() -> Topic {
    let (_, topics) = bundles::corpus_whole();
    topics[bundles::document(&topics, "question-shapes")].clone()
}

fn fenced(body: &str) -> String {
    format!("Разбор ответа.\n\n```json\n{body}\n```\n")
}

fn read(text: &str) -> Verdict {
    parse(text, &topic()).unwrap()
}

fn fail(text: &str) -> VerdictError {
    parse(text, &topic()).unwrap_err()
}

#[test]
fn the_last_block_of_the_answer_is_the_verdict() {
    let earlier = WHOLE.replace("\"verdict\": \"partial\"", "\"verdict\": \"fail\"");

    let verdict = read(&format!("{}\n{}", fenced(&earlier), fenced(WHOLE)));

    assert_eq!(verdict.result, Result::Partial);
}

#[test]
fn a_block_without_a_fence_is_still_found() {
    let verdict = read(&format!("Разбор.\n\n{WHOLE}\n"));

    assert_eq!(verdict.result, Result::Partial);
}

#[test]
fn the_last_of_two_unfenced_blocks_wins() {
    let earlier = WHOLE.replace("\"verdict\": \"partial\"", "\"verdict\": \"fail\"");

    let verdict = read(&format!("Черновик.\n{earlier}\n\nИтог.\n{WHOLE}\n"));

    assert_eq!(verdict.result, Result::Partial);
}

#[test]
fn a_fenced_block_wins_over_braces_in_the_prose_after_it() {
    let verdict = read(&format!(
        "{}\nДальше жду {{ответа}} по практике.\n",
        fenced(WHOLE)
    ));

    assert_eq!(verdict.topic_id, "question-shapes");
}

#[test]
fn a_text_without_any_json_is_no_verdict() {
    let error = fail("Зачёт прошёл, всё хорошо.");

    assert!(matches!(error, VerdictError::NoJson));
}

#[test]
fn a_block_that_is_not_json_is_reported_as_malformed() {
    let error = fail(&fenced("{\"topic_id\": \"cycle-a\", }{"));

    assert!(matches!(error, VerdictError::Malformed { .. }));
}

#[test]
fn a_verdict_without_per_question_is_rejected() {
    let text = WHOLE.replace("\"per_question\"", "\"per_answers\"");

    let error = fail(&fenced(&text));

    assert!(matches!(error, VerdictError::NoAnswers));
}

#[test]
fn a_verdict_whose_per_question_is_empty_is_rejected_as_well() {
    let text = "{\"topic_id\": \"question-shapes\", \"verdict\": \"pass\", \"per_question\": []}";

    let error = fail(&fenced(text));

    assert!(matches!(error, VerdictError::NoAnswers));
}

#[test]
fn a_verdict_about_another_topic_names_both_topics() {
    let text = WHOLE.replace("\"question-shapes\"", "\"cycle-b\"");

    let error = fail(&fenced(&text));

    let VerdictError::WrongTopic { expected, found } = &error else {
        panic!("a foreign topic is expected to be refused: {error}");
    };
    assert_eq!(expected, "question-shapes");
    assert_eq!(found, "cycle-b");
    assert!(error.to_string().contains("cycle-b"));
}

#[test]
fn a_verdict_without_a_result_of_its_own_is_rejected() {
    let text = WHOLE.replace("\"verdict\": \"partial\",", "");

    let error = fail(&fenced(&text));

    assert!(matches!(&error, VerdictError::Missing { field } if field == "verdict"));
}

#[test]
fn a_verdict_that_names_no_topic_is_rejected() {
    let text = WHOLE.replace("\"topic_id\": \"question-shapes\",", "");

    let error = fail(&fenced(&text));

    assert!(matches!(&error, VerdictError::Missing { field } if field == "topic_id"));
}

#[test]
fn a_whole_verdict_is_read_field_by_field() {
    let verdict = read(&fenced(WHOLE));

    assert_eq!(verdict.topic_id, "question-shapes");
    assert_eq!(verdict.result, Result::Partial);
    assert_eq!(verdict.date.as_deref(), Some("2026-07-27"));
    assert_eq!(verdict.model.as_deref(), Some("opus"));
    assert!(verdict.hinted);
    assert!(!verdict.practice_accepted);
    assert_eq!(verdict.failed_checks, ["a1"]);
    assert_eq!(verdict.gaps, ["не держит порядок"]);
    assert_eq!(verdict.calibration.as_deref(), Some("сказал про обход"));
    assert_eq!(verdict.notes, ["просил подсказку"]);
    assert_eq!(verdict.next_action, Some(NextAction::RetryFailed));
    assert_eq!(verdict.retry_after_days, Some(3));
    assert!(verdict.missing.is_empty());
}

#[test]
fn every_answer_carries_its_outcome_quote_and_missed_signals() {
    let verdict = read(&fenced(WHOLE));

    let [answer] = verdict.per_question.as_slice() else {
        panic!("one answer is expected");
    };
    assert_eq!(answer.id, "q1");
    assert_eq!(answer.outcome, Outcome::Miss);
    assert_eq!(answer.quote.as_deref(), Some("ну, зависит"));
    assert_eq!(answer.missed, ["порядок обхода"]);
    assert!(!answer.signal_extension);
}

#[test]
fn the_raw_block_is_kept_as_it_arrived() {
    let verdict = read(&fenced(WHOLE));

    assert_eq!(verdict.raw.trim(), WHOLE);
}

#[test]
fn what_the_verdict_lacks_is_listed_and_the_rest_is_applied() {
    let text = "{\"topic_id\": \"question-shapes\", \"verdict\": \"pass\",
        \"per_question\": [{\"id\": \"q1\", \"result\": \"ok\"}]}";

    let verdict = read(&fenced(text));

    assert_eq!(verdict.result, Result::Pass);
    assert!(verdict.missing.contains(&"date".to_owned()));
    assert!(verdict.missing.contains(&"calibration".to_owned()));
    assert!(verdict.missing.contains(&"next_action".to_owned()));
    assert!(!verdict.missing.contains(&"verdict".to_owned()));
}

#[test]
fn an_answer_to_a_question_the_topic_does_not_have_is_reported_not_refused() {
    let text = WHOLE.replace("\"id\": \"q1\"", "\"id\": \"q9\"");

    let verdict = read(&fenced(&text));

    assert_eq!(verdict.unknown_questions, ["q9"]);
    assert_eq!(verdict.per_question.len(), 1);
}

#[test]
fn an_answer_of_a_question_the_topic_does_have_is_no_complaint() {
    let verdict = read(&fenced(WHOLE));

    assert!(verdict.unknown_questions.is_empty());
}
