#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use std::fs;

use support::programs::{plant, scratch};
use support::search::{fresh, home, retitle, shelf};
use tolearn_core::library::Library;
use tolearn_core::search::Index;

#[test]
fn untouched_programs_are_not_read_again() {
    let (_, library) = shelf("refresh-kept");
    let mut index = Index::default();
    let first = index.refresh(&library).unwrap();

    let second = index.refresh(&library).unwrap();

    assert_eq!(first.indexed, 2);
    assert_eq!(second.indexed, 0);
    assert_eq!(second.kept, 2);
    assert_eq!(second.dropped, 0);
}

#[test]
fn an_edited_stage_rereads_only_its_program() {
    let (data, library) = shelf("refresh-changed");
    let mut index = fresh(&library);

    retitle(&data, "Тарабарщина");
    let report = index.refresh(&library).unwrap();

    assert_eq!((report.indexed, report.kept), (1, 1));
    assert!(!index.find("тарабарщина", 10).is_empty());
    assert!(index.find("голоса чипа", 10).is_empty());
}

#[test]
fn a_new_program_is_read_alone() {
    let data = scratch("search-refresh-added");
    plant(&data, "examples/chiptune");
    let library = Library::at(&data);
    let mut index = fresh(&library);

    plant(&data, "fixtures/v2/valid/nes-dev");
    let report = index.refresh(&library).unwrap();

    assert_eq!((report.indexed, report.kept), (1, 1));
    assert!(!index.find("компоновщика", 10).is_empty());
}

#[test]
fn a_removed_program_leaves_the_index() {
    let (data, library) = shelf("refresh-dropped");
    let mut index = fresh(&library);

    fs::remove_dir_all(home(&data)).unwrap();
    let report = index.refresh(&library).unwrap();

    assert_eq!((report.indexed, report.dropped), (0, 1));
    assert!(index.find("голоса чипа", 10).is_empty());
}

#[test]
fn the_index_survives_saving_and_reading() {
    let (data, library) = shelf("refresh-saved");
    retitle(&data, "Абракадабра \"в кавычках\"\\ и слэш");
    fs::write(home(&data).join("заметка\t\u{1}\u{85}.txt"), "текст").unwrap();
    let saved = fresh(&library);
    let path = data.join("search.yaml");
    saved.save(&path).unwrap();

    let mut read = Index::read(&path);
    assert_eq!(read, saved);
    let report = read.refresh(&library).unwrap();

    assert_eq!(report.indexed, 0);
    assert_eq!(read.find("абракадабра", 10).len(), 1);
}

#[test]
fn a_v1_index_reads_as_empty_and_is_rebuilt() {
    let (data, library) = shelf("refresh-legacy");
    let path = data.join("search.yaml");
    let v1 = "schema: tolearn/search/v1\nsources:\n  - path: \"/bundle/topics/local-runtime.yaml\"\n    roadmap: \"llm-agents-base\"\n    modified: \"1\"\n    size: \"1\"\n    documents:\n      - kind: topic\n        topic: \"local-runtime\"\n        title: \"Локальный рантайм\"\n        text: \"\"\n";
    fs::write(&path, v1).unwrap();
    assert_eq!(Index::read(&path), Index::default());

    let relabelled = fresh(&library)
        .text()
        .replacen("tolearn/search/v2", "tolearn/search/v1", 1);
    fs::write(&path, relabelled).unwrap();
    let mut read = Index::read(&path);
    assert_eq!(read, Index::default());

    let report = read.refresh(&library).unwrap();
    assert_eq!(report.indexed, 2);
    assert!(!read.find("голоса чипа", 10).is_empty());
}

#[test]
fn an_index_from_another_build_reads_as_empty() {
    let (data, library) = shelf("refresh-build");
    let path = data.join("search.yaml");
    let build = format!("build: \"{}\"", env!("CARGO_PKG_VERSION"));
    let text = fresh(&library).text();
    assert!(text.contains(&build), "{text}");

    fs::write(&path, text.replacen(&build, "build: \"0.0.0\"", 1)).unwrap();

    assert_eq!(Index::read(&path), Index::default());
}

#[test]
fn a_damaged_index_reads_as_empty() {
    let data = scratch("search-refresh-garbage");
    let path = data.join("search.yaml");
    fs::write(&path, "не: [йaml").unwrap();

    assert_eq!(Index::read(&path), Index::default());
}

#[test]
fn a_missing_index_reads_as_empty() {
    let data = scratch("search-refresh-absent");

    assert_eq!(Index::read(&data.join("нет-такого.yaml")), Index::default());
}
