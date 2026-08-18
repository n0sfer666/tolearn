#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "registry gate: a panic here is the report"
)]

use std::path::{Path, PathBuf};

use tolearn_core::registry::{Program, Registry};

fn scratch(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("tolearn-registry-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn bundle(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn program(id: &str, path: &Path) -> Program {
    Program {
        id: id.to_owned(),
        title: format!("Программа {id}"),
        path: path.to_owned(),
        opened_at: None,
    }
}

fn kept(directory: &Path, registry: &Registry) -> Registry {
    let file = directory.join("registry.yaml");
    registry.save(&file).unwrap();
    Registry::read(&file).unwrap()
}

#[test]
fn a_registry_that_was_saved_reads_back_after_a_restart() {
    let directory = scratch("restart");
    let first = bundle(&directory, "one");
    let second = bundle(&directory, "two");
    let mut registry = Registry::default();
    registry.add(program("one", &first));
    registry.add(program("two", &second));

    let read = kept(&directory, &registry);

    let ids: Vec<&str> = read.programs().iter().map(|it| it.id.as_str()).collect();
    assert_eq!(ids, ["one", "two"]);
    assert_eq!(read.programs()[0].path, first);
    assert_eq!(read.programs()[0].title, "Программа one");
}

#[test]
fn a_registry_that_was_never_written_is_empty_rather_than_an_error() {
    let directory = scratch("absent");

    let read = Registry::read(&directory.join("registry.yaml")).unwrap();

    assert!(read.programs().is_empty());
}

#[test]
fn a_program_added_twice_keeps_one_record_with_the_newer_path() {
    let directory = scratch("twice");
    let first = bundle(&directory, "old");
    let second = bundle(&directory, "new");
    let mut registry = Registry::default();
    registry.add(program("one", &first));

    registry.add(program("one", &second));

    assert_eq!(registry.programs().len(), 1);
    assert_eq!(registry.programs()[0].path, second);
}

#[test]
fn a_program_that_was_forgotten_is_gone_and_its_neighbours_stay() {
    let directory = scratch("forget");
    let mut registry = Registry::default();
    for name in ["one", "two"] {
        registry.add(program(name, &bundle(&directory, name)));
    }

    registry.forget("one");

    let ids: Vec<&str> = registry
        .programs()
        .iter()
        .map(|it| it.id.as_str())
        .collect();
    assert_eq!(ids, ["two"]);
}

#[test]
fn opening_a_program_records_when_it_happened() {
    let directory = scratch("touch");
    let path = bundle(&directory, "one");
    let mut registry = Registry::default();
    registry.add(program("one", &path));

    registry.touch("one", "2026-07-27T18:40:00+03:00");

    let read = kept(&directory, &registry);
    assert_eq!(
        read.programs()[0].opened_at.as_deref(),
        Some("2026-07-27T18:40:00+03:00")
    );
}

#[test]
fn a_program_whose_folder_is_there_is_reachable() {
    let directory = scratch("reachable");
    let path = bundle(&directory, "one");
    let mut registry = Registry::default();
    registry.add(program("one", &path));

    let [listed] = registry.entries().try_into().unwrap();

    assert!(listed.reachable);
}

#[test]
fn a_program_whose_folder_went_away_is_marked_not_dropped() {
    let directory = scratch("unreachable");
    let path = bundle(&directory, "one");
    let mut registry = Registry::default();
    registry.add(program("one", &path));
    std::fs::remove_dir_all(&path).unwrap();

    let listed = registry.entries();

    assert_eq!(listed.len(), 1);
    assert!(!listed[0].reachable);
    assert_eq!(listed[0].program.id, "one");
}

#[test]
fn an_unreachable_program_survives_a_save_with_its_title() {
    let directory = scratch("survives");
    let path = bundle(&directory, "one");
    let mut registry = Registry::default();
    registry.add(program("one", &path));
    std::fs::remove_dir_all(&path).unwrap();

    let read = kept(&directory, &registry);

    assert_eq!(read.programs().len(), 1);
    assert_eq!(read.programs()[0].title, "Программа one");
    assert!(!read.entries()[0].reachable);
}

#[test]
fn a_title_with_quotes_and_a_path_with_spaces_come_back_as_they_were() {
    let directory = scratch("quotes");
    let path = bundle(&directory, "две папки");
    let mut registry = Registry::default();
    registry.add(Program {
        id: "one".to_owned(),
        title: "Агенты: \"база\", C:\\путь".to_owned(),
        path: path.clone(),
        opened_at: None,
    });

    let read = kept(&directory, &registry);

    assert_eq!(read.programs()[0].title, "Агенты: \"база\", C:\\путь");
    assert_eq!(read.programs()[0].path, path);
}

#[test]
fn a_registry_file_that_is_not_a_registry_is_an_error_with_a_reason() {
    let directory = scratch("broken");
    let file = directory.join("registry.yaml");
    std::fs::write(&file, "schema: tolearn/registry/v1\nprograms: 17\n").unwrap();

    let error = Registry::read(&file).unwrap_err();

    assert!(!error.to_string().is_empty());
}

#[test]
fn the_order_of_the_programs_is_the_order_they_were_added_in() {
    let directory = scratch("order");
    let mut registry = Registry::default();
    for name in ["c", "a", "b"] {
        registry.add(program(name, &bundle(&directory, name)));
    }

    let read = kept(&directory, &registry);

    let ids: Vec<&str> = read.programs().iter().map(|it| it.id.as_str()).collect();
    assert_eq!(ids, ["c", "a", "b"]);
}
