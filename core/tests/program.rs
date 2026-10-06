#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "program reading: a panic here is the report"
)]

mod support;

use tolearn_core::Hours;
use tolearn_core::block::Kind;
use tolearn_core::program::{self, Volatility};
use tolearn_core::stage;
use tolearn_core::yaml::ParseFailure;

use support::{line_of, read};

const PROGRAM: &str = "examples/chiptune/program.yaml";
const STAGE: &str = "examples/chiptune/stages/voices.yaml";
const NODE: &str = "fixtures/v2/valid/nes-dev/program.yaml";
const FIRST_ROM: &str = "fixtures/v2/valid/nes-dev/children/b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56/children/e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47/stages/first-rom.yaml";

#[test]
fn the_reference_program_is_read_with_its_map_and_sources() {
    let program = program::parse(&read(PROGRAM)).unwrap();

    assert_eq!(program.uuid, "3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84");
    assert_eq!(program.slug, "chiptune");
    assert_eq!(program.generation.locale, "ru");
    assert_eq!(program.generation.volatility, Volatility::Stable);
    let rows: Vec<(&str, Hours)> = program
        .map
        .stages
        .iter()
        .map(|row| (row.id.as_str(), row.hours))
        .collect();
    assert_eq!(
        rows,
        [
            ("voices", Hours { min: 2, max: 3 }),
            ("envelope", Hours { min: 2, max: 4 }),
            ("first-track", Hours { min: 3, max: 4 }),
        ]
    );
    assert!(program.map.children.is_empty());
    let book = &program.sources.books[0];
    assert_eq!(book.authors, ["Karen Collins"]);
    assert_eq!(book.isbn, "9780262033787");
    assert_eq!(book.checked_at, "2026-09-11");
    assert_eq!(
        program.sources.pages[0].url,
        "https://www.nesdev.org/wiki/APU"
    );
}

#[test]
fn the_reference_stage_is_read_with_its_blocks_practice_and_questions() {
    let stage = stage::parse(&read(STAGE)).unwrap();

    assert_eq!(stage.id, "voices");
    let kinds: Vec<Kind> = stage.blocks.iter().map(|block| block.kind).collect();
    assert_eq!(
        kinds,
        [
            Kind::Heading,
            Kind::Paragraph,
            Kind::Diagram,
            Kind::Paragraph,
            Kind::Image,
            Kind::Paragraph,
            Kind::Code,
            Kind::Callout,
            Kind::Paragraph,
        ]
    );
    let image = &stage.blocks[4];
    assert_eq!(image.asset.as_deref(), Some("assets/pulse-wave.png"));
    assert_eq!(image.license.as_deref(), Some("CC0-1.0"));
    assert_eq!(image.attribution.as_deref(), Some("toLearn contributors"));
    assert_eq!(stage.blocks[6].lang.as_deref(), Some("python"));
    assert_eq!(stage.practice.task.len(), 2);
    let practice = &stage.practice;
    assert!(
        practice
            .constraints
            .iter()
            .chain(&practice.acceptance)
            .all(|check| check.check.is_none())
    );
    assert_eq!(stage.questions.len(), 4);
    assert_eq!(stage.questions[2].id, "q3");
}

#[test]
fn a_check_command_is_read_where_the_practice_has_one() {
    let stage = stage::parse(&read(FIRST_ROM)).unwrap();

    assert_eq!(
        stage.practice.constraints[0].check.as_deref(),
        Some("ca65 --version")
    );
    assert_eq!(
        stage.practice.task[5].source.as_deref(),
        Some("https://github.com/n0sfer666/tolearn")
    );
}

#[test]
fn a_program_of_another_major_is_refused() {
    let source = read(PROGRAM).replace("tolearn/program/1", "tolearn/program/2");

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "schema");
    assert_eq!(error.line(), 1);
}

#[test]
fn a_stage_written_against_the_program_schema_is_refused() {
    let source = read(STAGE).replace("tolearn/stage/1", "tolearn/program/1");

    let error = stage::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "schema");
}

#[test]
fn a_stage_row_that_is_no_slug_is_refused_before_it_becomes_a_path() {
    let source = read(PROGRAM).replace("- id: envelope", "- id: ../envelope");

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "map.stages[1].id");
    assert_eq!(error.line(), line_of(&source, "../envelope"));
}

#[test]
fn a_child_row_that_is_no_uuid_is_refused_before_it_becomes_a_path() {
    let source = read(NODE).replace(
        "- uuid: b25e8f13-6d47-4a09-a3c1-9e7f2d4b8c56",
        "- uuid: ../b25e8f13",
    );

    let error = program::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "map.children[0].uuid");
}

#[test]
fn a_block_of_an_unknown_kind_is_refused_where_it_stands() {
    let source = read(STAGE).replacen("kind: heading", "kind: video", 1);

    let error = stage::parse(&source).unwrap_err();

    assert_eq!(error.failure(), ParseFailure::UnknownValue);
    assert_eq!(error.path(), "blocks[0].kind");
    assert_eq!(error.line(), line_of(&source, "kind: video"));
}
