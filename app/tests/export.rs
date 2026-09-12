#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf, TOOLS, original};
use tolearn_app::ipc::IpcError;

fn export(shelf: &Shelf, program: &str, node: &str, folder: &Path) -> Result<Value, IpcError> {
    shelf.ask(
        "export",
        json!({
            "program": program,
            "node": node,
            "folder": folder.display().to_string(),
        }),
    )
}

fn listing(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_program_lands_in_a_folder_named_by_its_slug() {
    let shelf = Shelf::new("export-whole");
    shelf.shelved("examples/chiptune");
    let folder = shelf.incoming.join("выгрузка");
    fs::create_dir(&folder).unwrap();

    let out = export(&shelf, CHIPTUNE, "", &folder).unwrap();

    let target = folder.join("chiptune");
    assert_eq!(out["path"].as_str().unwrap(), target.display().to_string());
    assert_eq!(out["files"].as_u64().unwrap(), 4);
    let index = fs::read_to_string(target.join("index.md")).unwrap();
    assert!(index.starts_with("# Chiptune"), "{index}");
    assert_eq!(
        fs::read(target.join("assets/pulse-wave.png")).unwrap(),
        original("examples/chiptune/assets/pulse-wave.png")
    );
}

#[test]
fn a_subprogram_leaves_alone_with_its_own_assets() {
    let shelf = Shelf::new("export-branch");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    let out = export(&shelf, NES_DEV, ROM, &shelf.incoming).unwrap();

    let target = shelf.incoming.join("first-rom");
    assert_eq!(out["path"].as_str().unwrap(), target.display().to_string());
    let index = fs::read_to_string(target.join("index.md")).unwrap();
    assert!(!index.contains("../index.md"), "{index}");
    assert_eq!(
        fs::read(target.join("assets/pixel.png")).unwrap(),
        original(&format!(
            "fixtures/v2/valid/nes-dev/children/{TOOLS}/children/{ROM}/assets/pixel.png"
        ))
    );
}

#[test]
fn an_occupied_folder_is_refused_and_left_as_it_was() {
    let shelf = Shelf::new("export-occupied");
    shelf.shelved("examples/chiptune");
    let target = shelf.incoming.join("chiptune");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("note.txt"), "мои заметки").unwrap();

    let refused = export(&shelf, CHIPTUNE, "", &shelf.incoming).unwrap_err();

    assert_eq!(refused.code, "export.occupied");
    assert_eq!(listing(&target), ["note.txt"]);
}

#[test]
fn an_export_into_the_library_is_refused() {
    let shelf = Shelf::new("export-library");
    shelf.shelved("examples/chiptune");
    let programs = shelf.data.join("programs");
    let before = listing(&programs);

    for folder in [
        programs.clone(),
        programs.join(CHIPTUNE),
        programs.join(CHIPTUNE).join("нет").join("глубже"),
    ] {
        let refused = export(&shelf, CHIPTUNE, "", &folder).unwrap_err();

        assert_eq!(
            refused.code,
            "export.inside-library",
            "{}",
            folder.display()
        );
    }
    assert_eq!(listing(&programs), before);
    assert!(!programs.join(CHIPTUNE).join("нет").exists());
}

#[test]
fn a_folder_that_is_not_an_absolute_path_is_refused() {
    let shelf = Shelf::new("export-relative");
    shelf.shelved("examples/chiptune");

    for folder in ["", "выгрузка"] {
        let refused = export(&shelf, CHIPTUNE, "", Path::new(folder)).unwrap_err();

        assert_eq!(refused.code, "export.folder", "`{folder}`");
    }
    assert!(!Path::new("выгрузка").exists());
    assert!(!Path::new("chiptune").exists());
}
