#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use tolearn_core::roadmap::parse;
use tolearn_core::yaml::ParseFailure;

use support::{line_of, read};

const MINIMAL: &str = "fixtures/valid/roadmap/minimal.yaml";

#[test]
fn a_syntax_error_carries_the_place_it_broke() {
    let error = parse("schema: learning-roadmap/v1\n  id: broken\n").unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Syntax);
    assert_eq!(error.line(), 2);
    assert_eq!(error.column(), 5);
}

#[test]
fn a_type_mismatch_points_at_the_offending_value() {
    let source = read(MINIMAL).replace("weekly_hours: 1", "weekly_hours: сколько-нибудь");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "weekly_hours");
    assert_eq!(error.line(), line_of(&source, "weekly_hours"));
    assert_eq!(error.column(), 15);
    assert!(error.message().contains("integer"), "{error}");
}

#[test]
fn a_missing_field_is_named() {
    let source = read(MINIMAL).replace("locale: ru\n", "");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::MissingField);
    assert!(error.message().contains("locale"), "{error}");
}

#[test]
fn an_unknown_enum_value_is_rejected_with_its_path() {
    let source = read(MINIMAL).replace("priority: core", "priority: urgent");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "topics[0].priority");
    assert_eq!(error.line(), line_of(&source, "priority: urgent"));
    assert!(error.message().contains("urgent"), "{error}");
}

#[test]
fn an_error_deep_in_the_document_carries_the_whole_path() {
    let source = read(MINIMAL).replace("stable: 730", "stable: 0");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
    assert_eq!(error.path(), "defaults.revalidate_after_days.stable");
    assert_eq!(error.line(), line_of(&source, "stable: 0"));
    assert!(error.message().contains("not below 1"), "{error}");
}

#[test]
fn a_number_too_big_for_the_field_says_so() {
    let source = read(MINIMAL).replace("weekly_hours: 1", "weekly_hours: 5000000000");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
    assert_eq!(error.path(), "weekly_hours");
    assert!(error.message().contains("outside the range"), "{error}");
}

#[test]
fn a_key_that_is_not_a_string_points_at_the_key() {
    let source = read(MINIMAL).replace("version_pins: {}", "version_pins:\n  ollama: v1\n  17: v2");

    let error = parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.path(), "version_pins");
    assert_eq!(error.line(), line_of(&source, "17: v2"));
}

#[test]
fn a_file_holds_exactly_one_document() {
    let minimal = read(MINIMAL);

    let two = parse(&format!("{minimal}---\n{minimal}")).unwrap_err();
    let none = parse("").unwrap_err();

    assert_eq!(two.failure(), ParseFailure::DocumentCount);
    assert_eq!(none.failure(), ParseFailure::DocumentCount);
    assert!(two.message().contains("holds more"), "{two}");
    assert!(none.message().contains("holds none"), "{none}");
}

#[test]
fn every_error_carries_a_machine_code_and_a_place() {
    let error = parse(&read(MINIMAL).replace("locale: ru\n", "")).unwrap_err();

    assert_eq!(error.code(), "yaml.missing-field");
    assert!(
        error.to_string().starts_with("1:1: yaml.missing-field: "),
        "{error}"
    );
}
