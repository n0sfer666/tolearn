#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "parser gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;

use tolearn_core::progress::{Document, Format, parse, save};

use support::read;

fn scratch(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("tolearn-{name}-{}", std::process::id()));
    if directory.exists() {
        std::fs::remove_dir_all(&directory).unwrap();
    }
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn history(format: Format) -> Document {
    Document::read(
        &read("fixtures/valid/progress/attempts-history.yaml"),
        format,
    )
    .unwrap()
}

#[test]
fn a_saved_document_reads_back_from_disk_unchanged() {
    let directory = scratch("saved");
    let path = directory.join("progress.yaml");
    let document = history(Format::Yaml);

    save(&path, &document).unwrap();

    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(written, document.text());
    assert_eq!(&parse(&written).unwrap(), document.progress());
}

#[test]
fn saving_twice_leaves_the_second_version_and_no_leftovers() {
    let directory = scratch("twice");
    let path = directory.join("progress.yaml");
    let mut document = history(Format::Yaml);

    save(&path, &document).unwrap();
    document
        .push_attempt(
            "openai-compatible-api",
            &document.progress().state("local-runtime").unwrap().attempts[0].clone(),
        )
        .unwrap();
    save(&path, &document).unwrap();

    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(written, document.text());
    let files: Vec<String> = std::fs::read_dir(&directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(files, ["progress.yaml"]);
}

#[test]
fn the_file_that_was_there_survives_a_save_that_cannot_finish() {
    let directory = scratch("kept");
    let path = directory.join("progress.yaml");
    save(&path, &history(Format::Yaml)).unwrap();
    let before = std::fs::read_to_string(&path).unwrap();

    let error = save(
        &directory.join("gone").join("progress.yaml"),
        &history(Format::Yaml),
    );

    assert!(error.is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
}

#[test]
#[cfg(unix)]
fn a_file_that_forbids_writing_is_replaced_all_the_same() {
    use std::os::unix::fs::PermissionsExt;

    let directory = scratch("readonly");
    let path = directory.join("progress.yaml");
    std::fs::write(&path, "schema: nothing\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();

    let document = history(Format::Yaml);
    save(&path, &document).unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), document.text());
}
