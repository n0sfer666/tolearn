#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::topic::parse;
use tolearn_core::yaml::ParseFailure;

use support::{line_of, read};

const MINIMAL: &str = "fixtures/valid/topic/minimal-topic.yaml";
const REFERENCE: &str = "examples/llm-agents-base/topics/local-runtime.yaml";
const THREE_NUMBERS: &str = "fixtures/broken/topic/maxItems__est-hours-with-three-numbers.yaml";

#[test]
fn a_topic_that_is_not_a_topic_is_rejected() {
    let roadmap = read("examples/llm-agents-base/roadmap.yaml");

    let error = parse(&roadmap).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::MissingField);
}

#[test]
fn an_unknown_material_type_carries_its_index() {
    let source = read(REFERENCE).replace("    type: article", "    type: podcast");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "materials[3].type");
    assert_eq!(error.line(), line_of(&source, "type: podcast"));
    assert!(error.message().contains("podcast"), "{error}");
}

#[test]
fn a_check_deep_inside_acceptance_reports_its_whole_path() {
    let source = read(MINIMAL).replace("    - id: a1", "    - id: 1");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "practice.acceptance[0].id");
    assert_eq!(error.line(), line_of(&source, "- id: 1"));
}

#[test]
fn a_field_missing_inside_practice_is_named_with_its_owner() {
    let source = read(MINIMAL).replace("  time_box_min: 30\n", "");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::MissingField);
    assert_eq!(error.path(), "practice");
    assert!(error.message().contains("time_box_min"), "{error}");
}

#[test]
fn a_null_where_text_is_required_is_rejected() {
    let source = read(MINIMAL).replace("    - id: a1", "    - id: null");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "practice.acceptance[0].id");
}

#[test]
fn an_optional_field_still_has_to_be_present() {
    let source = read(MINIMAL).replace("  fallback: null\n", "");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::MissingField);
    assert!(error.message().contains("fallback"), "{error}");
}

#[test]
fn est_hours_holds_exactly_two_numbers() {
    let source = read(THREE_NUMBERS);
    assert!(
        source.contains("est_hours: [1, 2, 3]"),
        "the fixture lost its third number"
    );

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "est_hours");
    assert_eq!(error.line(), line_of(&source, "est_hours"));
}

#[test]
fn every_count_of_the_topic_starts_at_one() {
    let counts = [
        ("stage: 1", "stage"),
        ("est_hours: [1, 2]", "est_hours[0]"),
        ("revalidate_after_days: 730", "revalidate_after_days"),
        ("time_box_min: 30", "practice.time_box_min"),
        ("max_exchanges: 3", "exam.max_exchanges"),
    ];

    for (line, path) in counts {
        let (field, _) = line.split_once(':').unwrap();
        let zero = match field {
            "est_hours" => "est_hours: [0, 2]".to_owned(),
            _ => format!("{field}: 0"),
        };
        let source = read(MINIMAL).replace(line, &zero);
        assert!(source.contains(&zero), "the fixture lost `{line}`");

        let error = parse(&source).unwrap_err();

        assert_eq!(error.failure(), ParseFailure::OutOfRange, "{path}");
        assert_eq!(error.path(), path);
        assert!(error.message().contains("not below 1"), "{path}: {error}");
    }
}

#[test]
fn an_optional_field_rejects_a_value_that_is_not_text() {
    let source = read(MINIMAL).replace("  fallback: null", "  fallback: [1, 2]");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "practice.fallback");
}
