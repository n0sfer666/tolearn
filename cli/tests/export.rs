#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

#[allow(
    dead_code,
    reason = "the export gate takes only its part of the shared support module"
)]
mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::{code, copy, json, scratch, stderr, stdout, tolearn};

fn repo(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(relative)
}

fn chiptune() -> PathBuf {
    repo("examples/chiptune")
}

fn text(path: &Path) -> String {
    path.to_str().unwrap().to_owned()
}

#[test]
fn an_export_writes_a_folder_of_pages() {
    let into = scratch("export-folder").join("chiptune");

    let out = tolearn(&["export", &text(&chiptune()), &text(&into), "--json"]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(json(&out)["files"], 4);
    let index = fs::read_to_string(into.join("index.md")).unwrap();
    assert!(index.starts_with("# Chiptune"), "{index}");
    assert!(into.join("01-voices.md").is_file());
    assert!(into.join("assets/pulse-wave.png").is_file());
}

#[test]
fn an_export_names_the_folder_it_wrote() {
    let into = scratch("export-said").join("chiptune");

    let out = tolearn(&["export", &text(&chiptune()), &text(&into)]);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("файлов: 4"), "{}", stdout(&out));
}

#[test]
fn an_export_into_the_program_is_refused() {
    let program = scratch("export-inside").join("chiptune");
    copy(&chiptune(), &program);

    let out = tolearn(&[
        "export",
        &text(&program),
        &text(&program.join("out").join("deeper")),
    ]);

    assert_ne!(code(&out), 0);
    assert!(
        stderr(&out).contains("каталоге программы"),
        "{}",
        stderr(&out)
    );
    assert!(!program.join("out").exists(), "экспорт записан в программу");
}

#[test]
fn an_export_into_another_program_is_refused() {
    let other = scratch("export-other").join("nes-dev");
    copy(&repo("fixtures/v2/valid/nes-dev"), &other);

    let out = tolearn(&["export", &text(&chiptune()), &text(&other.join("out"))]);

    assert_ne!(code(&out), 0);
    assert!(
        stderr(&out).contains("каталоге программы"),
        "{}",
        stderr(&out)
    );
    assert!(!other.join("out").exists(), "экспорт записан в программу");
}

#[test]
fn an_occupied_folder_is_refused_and_left_alone() {
    let into = scratch("export-occupied");
    fs::write(into.join("note.txt"), "мои заметки").unwrap();

    let out = tolearn(&["export", &text(&chiptune()), &text(&into)]);

    assert_ne!(code(&out), 0);
    assert!(stderr(&out).contains("export.occupied"), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(into.join("note.txt")).unwrap(),
        "мои заметки"
    );
    assert!(!into.join("index.md").exists());
}

#[test]
fn an_export_without_a_folder_says_so() {
    let out = tolearn(&["export", &text(&chiptune())]);

    assert_eq!(code(&out), 2);
    assert!(stderr(&out).contains("не назвал папку"), "{}", stderr(&out));
}
