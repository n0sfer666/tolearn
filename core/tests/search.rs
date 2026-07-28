#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use tolearn_core::search::{Index, Kind};

struct Case {
    root: PathBuf,
    bundle: PathBuf,
    notes: PathBuf,
}

fn case(name: &str) -> Case {
    let root = std::env::temp_dir().join(format!("tolearn-search-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let bundle = root.join("bundle");
    copy(&support::root().join("examples/llm-agents-base"), &bundle);
    let _ = std::fs::remove_file(bundle.join("roadmap.json"));
    let notes = root.join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    Case {
        root,
        bundle,
        notes,
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy(&path, &target);
        } else if path.extension().is_some_and(|kind| kind == "yaml") {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

fn note(case: &Case, name: &str, topic: &str, body: &str) -> PathBuf {
    let path = case.notes.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        format!("---\ntolearn:\n  roadmap: llm-agents-base\n  topic: {topic}\n---\n\n{body}\n"),
    )
    .unwrap();
    path
}

fn fresh(case: &Case) -> Index {
    let mut index = Index::default();
    index.refresh(&case.bundle, &case.notes).unwrap();
    index
}

#[test]
fn тема_находится_по_названию() {
    let case = case("topic");
    let index = fresh(&case);

    let hits = index.find("локальный рантайм", 10);

    let first = hits.first().expect("тема должна найтись");
    assert_eq!(first.kind, Kind::Topic);
    assert_eq!(first.topic, "local-runtime");
    assert_eq!(first.roadmap, "llm-agents-base");
}

#[test]
fn материал_находится_по_заголовку() {
    let case = case("material");
    let index = fresh(&case);

    let hits = index.find("context length ollama", 10);

    let first = hits.first().expect("материал должен найтись");
    assert_eq!(first.kind, Kind::Material);
    assert_eq!(first.topic, "local-runtime");
}

#[test]
fn конспект_находится_по_тексту() {
    let case = case("note");
    note(
        &case,
        "своя/заметка.md",
        "local-runtime",
        "Проверил на своём железе",
    );
    let index = fresh(&case);

    let hits = index.find("своём железе", 10);

    let first = hits.first().expect("конспект должен найтись");
    assert_eq!(first.kind, Kind::Note);
    assert_eq!(first.topic, "local-runtime");
    assert!(first.snippet.contains("железе"), "{}", first.snippet);
}

#[test]
fn нетронутые_файлы_не_перечитываются() {
    let case = case("kept");
    note(&case, "заметка.md", "local-runtime", "Первый текст");
    let mut index = Index::default();
    let first = index.refresh(&case.bundle, &case.notes).unwrap();

    let second = index.refresh(&case.bundle, &case.notes).unwrap();

    assert_eq!(first.indexed, 8);
    assert_eq!(second.indexed, 0);
    assert_eq!(second.kept, first.indexed);
    assert_eq!(second.dropped, 0);
}

#[test]
fn правка_перечитывает_только_изменённый_файл() {
    let case = case("changed");
    note(&case, "заметка.md", "local-runtime", "Абракадабра первая");
    let mut index = Index::default();
    index.refresh(&case.bundle, &case.notes).unwrap();

    note(&case, "заметка.md", "local-runtime", "Тарабарщина вторая");
    let report = index.refresh(&case.bundle, &case.notes).unwrap();

    assert_eq!(report.indexed, 1);
    assert!(!index.find("тарабарщина", 10).is_empty());
    assert!(index.find("абракадабра", 10).is_empty());
}

#[test]
fn удалённый_конспект_уходит_из_индекса() {
    let case = case("dropped");
    let path = note(
        &case,
        "заметка.md",
        "local-runtime",
        "Уникальное слово абракадабра",
    );
    let mut index = Index::default();
    index.refresh(&case.bundle, &case.notes).unwrap();

    std::fs::remove_file(&path).unwrap();
    let report = index.refresh(&case.bundle, &case.notes).unwrap();

    assert_eq!(report.dropped, 1);
    assert_eq!(report.indexed, 0);
    assert!(index.find("абракадабра", 10).is_empty());
}

#[test]
fn чужой_конспект_не_попадает_в_выдачу() {
    let case = case("foreign");
    std::fs::write(
        case.notes.join("чужой.md"),
        "---\ntolearn:\n  roadmap: другая-программа\n  topic: local-runtime\n---\n\nабракадабра\n",
    )
    .unwrap();
    let index = fresh(&case);

    assert!(index.find("абракадабра", 10).is_empty());
}

#[test]
fn индекс_переживает_запись_и_чтение() {
    let case = case("saved");
    note(
        &case,
        "заметка.md",
        "local-runtime",
        "Уникальное слово абракадабра",
    );
    let saved = fresh(&case);
    let path = case.root.join("search.yaml");
    saved.save(&path).unwrap();

    let mut read = Index::read(&path).unwrap();
    let report = read.refresh(&case.bundle, &case.notes).unwrap();

    assert_eq!(report.indexed, 0);
    assert_eq!(read.find("абракадабра", 10).len(), 1);
}

#[test]
fn отсутствующий_индекс_читается_как_пустой() {
    let case = case("absent");

    let index = Index::read(&case.root.join("нет-такого.yaml")).unwrap();

    assert!(index.find("рантайм", 10).is_empty());
}

#[test]
fn запрос_требует_все_слова() {
    let case = case("all-words");
    note(&case, "заметка.md", "local-runtime", "абракадабра про кэш");
    let index = fresh(&case);

    assert_eq!(index.find("абракадабра кэш", 10).len(), 1);
    assert!(index.find("абракадабра тарабарщина", 10).is_empty());
}

#[test]
fn пустой_запрос_ничего_не_находит() {
    let case = case("empty-query");
    let index = fresh(&case);

    assert!(index.find("   ", 10).is_empty());
}

#[test]
fn выдача_обрезается_до_предела() {
    let case = case("limit");
    let index = fresh(&case);

    assert_eq!(index.find("модель", 3).len(), 3);
}

#[test]
fn битая_тема_не_валит_обход() {
    let case = case("broken");
    std::fs::write(case.bundle.join("topics/local-runtime.yaml"), "не: [йaml").unwrap();

    let index = fresh(&case);

    assert!(
        index
            .find("локальный рантайм", 10)
            .iter()
            .all(|hit| hit.topic != "local-runtime")
    );
    assert!(!index.find("фолбэки", 10).is_empty());
}
