#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::repository;
use support::shelf::{CHIPTUNE, Shelf};

#[test]
fn an_empty_library_lists_nothing() {
    let shelf = Shelf::new("empty");

    assert_eq!(shelf.library(), json!({ "programs": [], "refused": [] }));
}

#[test]
fn an_imported_package_is_shelved_with_its_goal_and_map_hours() {
    let shelf = Shelf::new("shelved");

    let answer = shelf.shelved("examples/chiptune");

    assert_eq!(answer["uuid"], CHIPTUNE);
    assert_eq!(answer["title"], "Chiptune: музыка звукового чипа NES");
    assert_eq!(answer["copy_of"], Value::Null);
    let listed = shelf.library();
    let card = &listed["programs"][0];
    assert_eq!(card["uuid"], CHIPTUNE);
    assert_eq!(card["title"], answer["title"]);
    assert!(
        card["goal"]
            .as_str()
            .unwrap()
            .starts_with("Написать и проиграть"),
        "{card}"
    );
    assert_eq!(card["hours"], json!({ "min": 7, "max": 11 }));
    assert_eq!(
        card["summary"],
        json!({ "passed": 0, "total": 1, "skipped": 0 })
    );
    assert_eq!(listed["refused"], json!([]));
}

#[test]
fn a_second_import_of_the_same_package_arrives_as_a_copy() {
    let shelf = Shelf::new("copy");
    let file = shelf.packed("examples/chiptune");

    shelf.import(&file).unwrap();
    let again = shelf.import(&file).unwrap();

    assert_eq!(again["copy_of"], CHIPTUNE);
    assert_ne!(again["uuid"], CHIPTUNE);
    assert_eq!(again["title"], "Chiptune: музыка звукового чипа NES");
    assert_eq!(shelf.library()["programs"].as_array().unwrap().len(), 2);
}

#[test]
fn a_v1_folder_or_archive_is_answered_as_v1_not_as_a_parse_error() {
    let shelf = Shelf::new("v1");
    let folder = shelf.incoming.join("rust-base");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("roadmap.yaml"), b"id: rust-base\n").unwrap();
    let mut paths = vec![folder];
    for name in ["bundle.zip", "bundle.tar.gz", "bundle.tgz", "BUNDLE.ZIP"] {
        let file = shelf.incoming.join(name);
        std::fs::write(&file, b"not an archive").unwrap();
        paths.push(file);
    }

    for path in paths {
        let error = shelf.import(&path).unwrap_err();
        assert_eq!(error.code, "package.v1", "{}", path.display());
        assert!(
            error.message.contains("v1 не открываются"),
            "{}",
            error.message
        );
    }
    assert!(!shelf.data.join("programs").exists());
}

#[test]
fn a_program_folder_and_a_foreign_file_are_refused_with_their_reason() {
    let shelf = Shelf::new("foreign");
    let note = shelf.incoming.join("notes.txt");
    std::fs::write(&note, "не пакет").unwrap();
    let downloads = shelf.incoming.join("downloads");
    std::fs::create_dir(&downloads).unwrap();

    let folder = shelf
        .import(&repository().join("examples/chiptune"))
        .unwrap_err();
    let foreign = shelf.import(&note).unwrap_err();
    let stray = shelf.import(&downloads).unwrap_err();

    assert_eq!(folder.code, "package.folder");
    assert!(
        folder.message.contains("tolearn pack"),
        "{}",
        folder.message
    );
    assert_eq!(foreign.code, "package.foreign");
    assert!(foreign.message.contains("notes.txt"), "{}", foreign.message);
    assert_eq!(stray.code, "package.foreign", "{}", stray.message);
    assert_eq!(shelf.library()["programs"], json!([]));
}

#[test]
fn a_broken_package_passes_its_own_code_through() {
    let shelf = Shelf::new("broken");
    let file = shelf.incoming.join("broken.tolearn");
    std::fs::write(&file, b"not a zip").unwrap();

    let error = shelf.import(&file).unwrap_err();

    assert!(error.code.starts_with("archive."), "{}", error.code);
    assert!(!error.message.is_empty());
    assert_eq!(shelf.library()["programs"], json!([]));
}

#[test]
fn a_broken_program_in_the_library_is_listed_as_refused() {
    let shelf = Shelf::new("refused");
    let directory = shelf.data.join("programs").join(CHIPTUNE);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("program.yaml"), "schema: nope\n").unwrap();

    let listed = shelf.library();

    assert_eq!(listed["programs"], json!([]));
    let refused = &listed["refused"][0];
    assert_eq!(refused["directory"], CHIPTUNE);
    assert!(!refused["code"].as_str().unwrap().is_empty(), "{refused}");
    assert!(
        !refused["message"].as_str().unwrap().is_empty(),
        "{refused}"
    );
}
