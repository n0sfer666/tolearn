#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "export gate: a panic here is the report"
)]

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use support::programs::{listing, scratch};
use tolearn_core::export::{Export, ExportError, Page, render, within, write};
use tolearn_core::program::load;

fn home() -> PathBuf {
    support::root().join("examples/chiptune")
}

fn exported(target: &Path) -> Result<usize, ExportError> {
    let home = home();
    write(&render(&load(&home).unwrap()), target, within(&home))
}

#[test]
fn an_export_lands_whole_in_an_absent_folder() {
    let room = scratch("export-absent");
    let target = room.join("out").join("chiptune");

    let written = exported(&target).unwrap();

    assert_eq!(written, 4);
    let index = fs::read_to_string(target.join("index.md")).unwrap();
    assert!(index.starts_with("# Chiptune"), "{index}");
    assert!(target.join("01-voices.md").is_file());
    for asset in ["assets/voices.svg", "assets/pulse-wave.png"] {
        assert_eq!(
            fs::read(target.join(asset)).unwrap(),
            fs::read(home().join(asset)).unwrap(),
            "{asset}"
        );
    }
    assert_eq!(listing(&room.join("out")), ["chiptune"]);
}

#[test]
fn an_empty_folder_is_taken() {
    let room = scratch("export-empty");
    let target = room.join("chiptune");
    fs::create_dir(&target).unwrap();

    exported(&target).unwrap();

    assert!(target.join("index.md").is_file());
    assert_eq!(listing(&room), ["chiptune"]);
}

#[test]
fn an_occupied_folder_is_refused_and_left_as_it_was() {
    let room = scratch("export-occupied");
    let target = room.join("chiptune");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("note.txt"), "мои заметки").unwrap();

    let error = exported(&target).unwrap_err();

    assert_eq!(error.code(), "export.occupied");
    assert_eq!(listing(&target), ["note.txt"]);
    assert_eq!(listing(&room), ["chiptune"]);
}

#[test]
fn a_file_in_place_of_the_folder_is_refused() {
    let room = scratch("export-file");
    let target = room.join("chiptune");
    fs::write(&target, "файл").unwrap();

    let error = exported(&target).unwrap_err();

    assert_eq!(error.code(), "export.occupied");
    assert_eq!(fs::read_to_string(&target).unwrap(), "файл");
}

#[test]
fn a_failure_midway_leaves_no_trace() {
    let room = scratch("export-midway");
    let page = |path: &str| Page {
        path: path.to_owned(),
        text: "текст\n".to_owned(),
    };
    let export = Export {
        pages: vec![page("a"), page("a/b.md")],
        assets: Vec::new(),
    };

    let error = write(&export, &room.join("out"), |_| Ok(Vec::new())).unwrap_err();

    assert_eq!(error.code(), "export.unwritable");
    assert!(listing(&room).is_empty(), "{:?}", listing(&room));
}

#[cfg(unix)]
#[test]
fn an_asset_linked_outside_the_program_stops_the_export_without_a_trace() {
    let room = scratch("export-foreign");
    let home = room.join("chiptune");
    support::programs::copy_tree(&self::home(), &home);
    fs::write(room.join("secret.svg"), "<svg/>").unwrap();
    fs::remove_file(home.join("assets/voices.svg")).unwrap();
    std::os::unix::fs::symlink(room.join("secret.svg"), home.join("assets/voices.svg")).unwrap();

    let error = write(
        &render(&load(&home).unwrap()),
        &room.join("out"),
        within(&home),
    )
    .unwrap_err();

    assert_eq!(error.code(), "export.asset");
    assert_eq!(listing(&room), ["chiptune", "secret.svg"]);
}
