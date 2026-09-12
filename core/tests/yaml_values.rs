#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "yaml layer: a panic here is the report"
)]

mod support;

use tolearn_core::yaml::ParseFailure;
use tolearn_core::{program, stage};

use support::{line_of, read};

const PROGRAM: &str = "examples/chiptune/program.yaml";
const STAGE: &str = "examples/chiptune/stages/voices.yaml";
const CHECKED: &str = "checked_at: 2026-09-11";

fn program_with(from: &str, to: &str) -> String {
    let source = read(PROGRAM);
    assert!(source.contains(from), "`{from}` is not in {PROGRAM}");
    source.replacen(from, to, 1)
}

fn checked_on(day: &str) -> Result<program::Program, tolearn_core::yaml::ParseError> {
    program::parse(&program_with(CHECKED, &format!("checked_at: \"{day}\"")))
}

#[test]
fn an_empty_string_is_refused_where_the_schema_asks_for_text() {
    let error = program::parse(&program_with("locale: ru", "locale: \"\"")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Empty);
    assert_eq!(error.code(), "yaml.empty");
    assert_eq!(error.path(), "generation.locale");
}

#[test]
fn a_string_of_spaces_counts_as_empty() {
    let error = program::parse(&program_with("locale: ru", "locale: \"   \"")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Empty);
}

#[test]
fn an_empty_block_text_is_refused_where_it_stands() {
    let source = read(STAGE).replacen("text: Пять каналов 2A03", "text: \"\"", 1);

    let error = stage::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::Empty);
    assert_eq!(error.path(), "blocks[0].text");
}

#[test]
fn a_date_that_is_not_a_date_is_refused() {
    let error = program::parse(&program_with(CHECKED, "checked_at: soon")).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::BadDate);
    assert_eq!(error.code(), "yaml.bad-date");
    assert_eq!(error.path(), "sources.books[0].checked_at");
}

#[test]
fn a_date_with_a_day_or_month_the_calendar_does_not_have_is_refused() {
    for day in [
        "2026-02-30",
        "2026-04-31",
        "2026-13-01",
        "2026-00-10",
        "2026-9-11",
    ] {
        let error = checked_on(day)
            .err()
            .unwrap_or_else(|| panic!("`{day}` is no date and is expected to be refused"));
        assert_eq!(error.failure(), ParseFailure::BadDate, "{day}");
    }
}

#[test]
fn the_twenty_ninth_of_february_is_a_date_in_a_leap_year_only() {
    checked_on("2024-02-29").unwrap();
    checked_on("2000-02-29").unwrap();

    for day in ["2026-02-29", "1900-02-29"] {
        let error = checked_on(day).unwrap_err();
        assert_eq!(error.failure(), ParseFailure::BadDate, "{day}");
    }
}

#[test]
fn a_number_below_what_the_schema_allows_is_refused() {
    let source = program_with("hours: [2, 3]", "hours: [0, 3]");

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
    assert_eq!(error.code(), "yaml.out-of-range");
    assert_eq!(error.path(), "map.stages[0].hours[0]");
    assert_eq!(error.line(), line_of(&source, "hours: [0, 3]"));
    assert!(error.message().contains("not below 1"), "{error}");
}

#[test]
fn a_number_too_big_for_the_field_says_so() {
    let source = program_with("hours: [2, 3]", "hours: [2, 5000000000]");

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::OutOfRange);
    assert_eq!(error.path(), "map.stages[0].hours[1]");
    assert!(error.message().contains("outside the range"), "{error}");
}

#[test]
fn a_type_mismatch_points_at_the_offending_value() {
    let source = program_with("locale: ru", "locale: [ru]");

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::WrongType);
    assert_eq!(error.code(), "yaml.wrong-type");
    assert_eq!(error.path(), "generation.locale");
    assert_eq!(error.line(), line_of(&source, "locale: [ru]"));
    assert_eq!(error.column(), 11);
    assert!(error.message().contains("string"), "{error}");
}
