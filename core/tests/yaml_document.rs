#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "yaml layer: a panic here is the report"
)]

mod support;

use tolearn_core::program;
use tolearn_core::yaml::ParseFailure;

use support::{line_of, read};

const PROGRAM: &str = "examples/chiptune/program.yaml";

#[test]
fn a_syntax_error_carries_the_place_it_broke() {
    let error = program::parse("schema: tolearn/program/1\n  id: broken\n").unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Syntax);
    assert_eq!(error.code(), "yaml.syntax");
    assert_eq!(error.line(), 2);
    assert_eq!(error.column(), 5);
}

#[test]
fn a_file_holds_exactly_one_document() {
    let source = read(PROGRAM);

    let two = program::parse(&format!("{source}---\n{source}")).unwrap_err();
    let none = program::parse("").unwrap_err();

    assert_eq!(two.failure(), ParseFailure::DocumentCount);
    assert_eq!(none.failure(), ParseFailure::DocumentCount);
    assert_eq!(two.code(), "yaml.document-count");
    assert!(two.message().contains("holds more"), "{two}");
    assert!(none.message().contains("holds none"), "{none}");
}

#[test]
fn a_key_that_repeats_in_the_same_mapping_reports_both_places() {
    let source = read(PROGRAM).replacen("slug: chiptune\n", "slug: chiptune\nslug: chiptune\n", 1);
    let first = line_of(&source, "slug:");

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::DuplicateKey);
    assert_eq!(error.code(), "yaml.duplicate-key");
    assert_eq!(error.line(), first + 1);
    assert!(
        error.message().contains(&format!("first on line {first}")),
        "{error}"
    );
}

#[test]
fn a_key_that_repeats_deeper_in_the_document_is_refused_where_it_repeats() {
    let source = read(PROGRAM).replacen("  locale: ru\n", "  locale: ru\n  locale: en\n", 1);

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::DuplicateKey);
    assert_eq!(error.line(), line_of(&source, "locale: en"));
    assert_eq!(error.path(), "generation");
}

#[test]
fn the_same_key_in_two_different_mappings_is_no_duplicate() {
    let source = read(PROGRAM);

    assert!(source.matches("title:").count() > 1, "{source}");
    program::parse(&source).unwrap();
}

#[test]
fn a_missing_field_is_named_with_a_machine_code_and_a_place() {
    let error = program::parse(&read(PROGRAM).replace("slug: chiptune\n", "")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::MissingField);
    assert_eq!(error.code(), "yaml.missing-field");
    assert!(error.message().contains("slug"), "{error}");
    assert!(
        error.to_string().starts_with("1:1: yaml.missing-field: "),
        "{error}"
    );
}

#[test]
fn a_byte_order_mark_before_the_first_key_is_no_part_of_it() {
    program::parse(&format!("\u{feff}{}", read(PROGRAM))).unwrap();
}
