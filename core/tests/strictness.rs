#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::yaml::ParseFailure;
use tolearn_core::{progress, roadmap, topic};

use support::{line_of, read};

const ROADMAP: &str = "examples/llm-agents-base/roadmap.yaml";
const TOPIC: &str = "examples/llm-agents-base/topics/local-runtime.yaml";
const PROGRESS: &str = "fixtures/valid/progress/attempts-history.yaml";

fn roadmap_with(field: &str, value: &str) -> String {
    replace(&read(ROADMAP), field, value)
}

fn topic_with(field: &str, value: &str) -> String {
    replace(&read(TOPIC), field, value)
}

fn progress_with(field: &str, value: &str) -> String {
    replace(&read(PROGRESS), field, value)
}

fn replace(source: &str, field: &str, value: &str) -> String {
    let line = line_of(source, &format!("{field}:"));
    source
        .lines()
        .enumerate()
        .map(|(index, text)| {
            if index + 1 == line {
                let cut = text.find(&format!("{field}:")).unwrap();
                format!("{}{field}: {value}", &text[..cut])
            } else {
                text.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_key_that_repeats_in_the_same_mapping_is_refused() {
    let source = read("fixtures/strict/roadmap/duplicate-key.yaml");

    let error = roadmap::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::DuplicateKey);
    assert_eq!(error.code(), "yaml.duplicate-key");
    assert!(error.message().contains("locale"), "{error}");
}

#[test]
fn a_repeated_key_reports_both_places_it_was_written() {
    let source = read("fixtures/strict/roadmap/duplicate-key.yaml");
    let first = source
        .lines()
        .position(|line| line.starts_with("locale:"))
        .unwrap()
        + 1;
    let second = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("locale:"))
        .nth(1)
        .unwrap()
        .0
        + 1;

    let error = roadmap::parse(&source).unwrap_err();

    assert_eq!(error.line(), second);
    assert!(
        error.message().contains(&first.to_string()),
        "the first `locale` is on line {first}, and the message does not say so: {error}"
    );
}

#[test]
fn a_key_that_repeats_deeper_in_the_document_is_refused_too() {
    let source = "\
schema: learning-roadmap/progress/v1
roadmap_id: minimal-program
topics:
  minimal-topic:
    status: todo
    status: passed
    attempts: []
    passed_at: null
    next_review_at: null
    gaps: []
";

    let error = progress::parse(source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::DuplicateKey);
    assert_eq!(error.line(), 6);
}

#[test]
fn the_same_key_in_two_different_mappings_is_no_duplicate() {
    let source = read(PROGRESS);

    assert!(source.matches("status:").count() > 1, "{source}");
    progress::parse(&source).unwrap();
}

#[test]
fn an_empty_string_is_refused_where_the_schema_asks_for_text() {
    let error = roadmap::parse(&roadmap_with("locale", "\"\"")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Empty);
    assert_eq!(error.code(), "yaml.empty");
    assert_eq!(error.path(), "locale");
}

#[test]
fn a_string_of_spaces_counts_as_empty() {
    let error = roadmap::parse(&roadmap_with("locale", "\"   \"")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Empty);
}

#[test]
fn the_raw_answer_may_be_empty_because_the_schema_allows_it() {
    let source = "\
schema: learning-roadmap/progress/v1
roadmap_id: minimal-program
topics:
  minimal-topic:
    status: todo
    attempts:
      - at: \"2026-07-20T11:05:00Z\"
        source: exam
        verdict: pass
        model: null
        hinted: false
        practice_accepted: true
        failed_checks: []
        per_question: []
        gaps: []
        calibration: null
        notes: []
        raw: \"\"
    passed_at: null
    next_review_at: null
    gaps: []
";

    let read = progress::parse(source).unwrap();

    assert_eq!(read.state("minimal-topic").unwrap().attempts[0].raw, "");
}

#[test]
fn a_date_that_is_not_a_date_is_refused() {
    let error = topic::parse(&topic_with("verified_at", "\"soon\"")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::BadDate);
    assert_eq!(error.code(), "yaml.bad-date");
    assert_eq!(error.path(), "verified_at");
}

#[test]
fn a_date_with_a_day_the_month_does_not_have_is_refused() {
    let error = topic::parse(&topic_with("verified_at", "\"2026-02-30\"")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::BadDate);
}

#[test]
fn the_twenty_ninth_of_february_is_a_date_in_a_leap_year_only() {
    topic::parse(&topic_with("verified_at", "\"2024-02-29\"")).unwrap();

    let error = topic::parse(&topic_with("verified_at", "\"2026-02-29\"")).unwrap_err();
    assert_eq!(error.failure(), ParseFailure::BadDate);
}

#[test]
fn a_date_where_the_schema_asks_for_a_moment_is_refused() {
    let error = progress_with("at", "\"2026-07-20\"");
    let error = progress::parse(&error).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::BadDate);
    assert!(error.message().contains("RFC 3339"), "{error}");
}

#[test]
fn a_moment_carries_its_offset() {
    progress::parse(&progress_with("at", "\"2026-07-20T11:05:00+03:00\"")).unwrap();

    let error = progress::parse(&progress_with("at", "\"2026-07-20T11:05:00\"")).unwrap_err();
    assert_eq!(error.failure(), ParseFailure::BadDate);
}

#[test]
fn an_offset_no_clock_could_show_is_refused() {
    for tail in ["+25:00", "+03:70", "+3:00", "+0300"] {
        let source = progress_with("at", &format!("\"2026-07-20T11:05:00{tail}\""));

        let error = progress::parse(&source)
            .err()
            .unwrap_or_else(|| panic!("`{tail}` is no offset and is expected to be refused"));
        assert_eq!(error.failure(), ParseFailure::BadDate, "{tail}");
    }
}

#[test]
fn a_clock_no_day_could_show_is_refused() {
    for clock in ["25:05:00", "11:60:00", "11:5:00"] {
        let source = progress_with("at", &format!("\"2026-07-20T{clock}Z\""));

        let error = progress::parse(&source)
            .err()
            .unwrap_or_else(|| panic!("`{clock}` is no clock and is expected to be refused"));
        assert_eq!(error.failure(), ParseFailure::BadDate, "{clock}");
    }
}

#[test]
fn a_date_that_may_be_missing_is_read_as_missing_and_not_as_a_broken_date() {
    let source = read(PROGRESS);

    let read = progress::parse(&source).unwrap();

    assert_eq!(read.state("structured-output").unwrap().passed_at, None);
    assert!(progress::parse(&progress_with("passed_at", "\"never\"")).is_err());
}

#[test]
fn a_number_below_what_the_schema_allows_is_refused() {
    let error = roadmap::parse(&roadmap_with("weekly_hours", "0")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
    assert_eq!(error.path(), "weekly_hours");
}

#[test]
fn a_stage_below_one_is_refused() {
    let error = topic::parse(&topic_with("stage", "0")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
}

#[test]
fn a_negative_number_of_days_is_refused() {
    let error = progress::parse(&progress_with("retry_after_days", "-1")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
}

#[test]
fn zero_days_of_waiting_is_allowed_because_the_schema_allows_it() {
    progress::parse(&progress_with("retry_after_days", "0")).unwrap();
}

#[test]
fn hours_that_run_backwards_are_left_to_the_validator() {
    let source = read("fixtures/strict/roadmap/hours-reversed.yaml");

    let map = roadmap::parse(&source).unwrap();

    let found = tolearn_core::bundle::validate(&map, &[]);

    assert!(
        found.iter().map(ToString::to_string).any(|violation| violation
            == "bundle.hours-reversed: `local-runtime` is estimated at 9 hours at the least and 4 at the most"),
        "{found:?}"
    );
}
