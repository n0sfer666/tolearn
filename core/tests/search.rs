#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::programs::{copy_tree, scratch};
use support::search::{CHIPTUNE, fresh, home, shelf};
use tolearn_core::library::Library;
use tolearn_core::search::{Hit, Index, Kind};

const NES_DEV: &str = "7a1d4e90-2c3b-4f58-8d6e-1b9a0c5e7f23";
const ROM: &str = "e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47";
const STRANGER: &str = "0b1c2d3e-4f50-4617-8293-a4b5c6d7e8f9";

fn found(name: &str, query: &str) -> Vec<Hit> {
    let (_, library) = shelf(name);
    fresh(&library).find(query, 10)
}

#[test]
fn a_stage_is_found_by_its_title() {
    let hits = found("stage", "голоса чипа");

    let first = hits.first().expect("этап должен найтись");
    assert_eq!(first.kind, Kind::Stage);
    assert_eq!(first.program, CHIPTUNE);
    assert_eq!(first.node, CHIPTUNE);
    assert_eq!(first.node_title, "Chiptune: музыка звукового чипа NES");
    assert_eq!(first.stage, "voices");
    assert_eq!(first.title, "Голоса чипа");
    assert_eq!(first.block, "");
}

#[test]
fn a_block_is_found_by_its_text_and_names_its_anchor() {
    let hits = found("block", "скважность периода");

    let first = hits.first().expect("блок должен найтись");
    assert_eq!(first.kind, Kind::Block);
    assert_eq!(first.stage, "voices");
    assert_eq!(first.title, "Голоса чипа");
    assert_eq!(first.block, "937ff0f4");
    assert!(first.snippet.contains("скважность"), "{}", first.snippet);
}

#[test]
fn a_practice_task_block_is_found_too() {
    let hits = found("task", "FamiStudio");

    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].block, "15bf109d");
}

#[test]
fn a_diagram_source_is_not_searchable() {
    assert!(found("diagram", "микшер").is_empty());
}

#[test]
fn a_stage_of_a_child_program_leads_into_its_node() {
    let hits = found("child", "компоновщика");

    let first = hits.first().expect("этап подпрограммы должен найтись");
    assert_eq!(first.program, NES_DEV);
    assert_eq!(first.node, ROM);
    assert_eq!(first.node_title, "Первый ROM в cc65");
    assert_eq!(first.stage, "linker");
}

#[test]
fn stage_titles_come_before_blocks_read_earlier() {
    let hits = found("stages-first", "перв");

    let first = hits.first().expect("этап должен найтись");
    assert_eq!(
        (first.kind, first.stage.as_str()),
        (Kind::Stage, "first-rom")
    );
    assert!(hits.iter().any(|hit| hit.block == "9bc182dd"), "{hits:?}");
}

#[test]
fn blocks_keep_reading_order() {
    let hits = found("order", "канал");

    let blocks: Vec<&str> = hits
        .iter()
        .filter(|hit| hit.kind == Kind::Block)
        .map(|hit| hit.block.as_str())
        .collect();
    assert_eq!(blocks[..2], ["68f347b0", "874672f8"], "{blocks:?}");
}

#[test]
fn answers_to_questions_are_not_searchable() {
    assert!(found("answers", "перевёрнутая по амплитуде").is_empty());
}

#[test]
fn a_query_needs_every_word_in_one_place() {
    assert!(!found("all-words", "импульсных мелодию").is_empty());
    assert!(found("all-words-none", "импульсных тарабарщина").is_empty());
}

#[test]
fn an_empty_query_finds_nothing() {
    assert!(found("empty-query", "   ").is_empty());
}

#[test]
fn the_list_is_cut_to_the_limit() {
    let (_, library) = shelf("limit");

    assert_eq!(fresh(&library).find("канал", 2).len(), 2);
}

#[test]
fn a_program_the_library_would_refuse_is_not_searchable() {
    let (data, library) = shelf("refused");
    fs::write(home(&data).join("program.yaml"), "не: [йaml").unwrap();
    let mut index = Index::default();

    let report = index.refresh(&library).unwrap();

    assert_eq!(report.indexed, 2);
    assert!(index.find("голоса чипа", 10).is_empty());
    assert!(!index.find("компоновщика", 10).is_empty());
}

#[test]
fn copies_outside_the_library_rules_are_skipped() {
    let (data, library) = shelf("strays");
    let programs = data.join("programs");
    for name in [STRANGER, "not-a-uuid", ".staging"] {
        copy_tree(&home(&data), &programs.join(name));
    }
    fs::write(programs.join("loose.yaml"), "текст").unwrap();
    let mut index = Index::default();

    let report = index.refresh(&library).unwrap();

    assert_eq!(report.indexed, 3);
    let hits = index.find("голоса чипа", 10);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].program, CHIPTUNE);
}

#[test]
fn an_empty_library_finds_nothing() {
    let library = Library::at(&scratch("search-nothing"));
    let mut index = Index::default();

    let report = index.refresh(&library).unwrap();

    assert_eq!(report.indexed, 0);
    assert!(index.find("канал", 10).is_empty());
}
