#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "stand gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::repository;
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf};
use tolearn_core::state;

const STAND: &str = "fixtures/v2/stand";

fn fixtures() -> Vec<(String, String)> {
    let dir = repository().join(STAND);
    let mut found: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .map(|path| {
            let program = path.file_stem().unwrap().to_string_lossy().into_owned();
            (program, std::fs::read_to_string(&path).unwrap())
        })
        .collect();
    found.sort();
    found
}

fn stood() -> Shelf {
    let shelf = Shelf::new("stand");
    shelf.shelved("examples/chiptune");
    shelf.shelved("fixtures/v2/valid/nes-dev");
    for (program, text) in fixtures() {
        let room = shelf.data.join("state").join(&program);
        std::fs::create_dir_all(&room).unwrap();
        std::fs::write(room.join("state.yaml"), text).unwrap();
    }
    shelf
}

fn marks(shelf: &Shelf, program: &str, node: &str) -> Vec<(String, String, Value)> {
    let shown = shelf
        .ask("node", json!({ "program": program, "node": node }))
        .unwrap();
    shown["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_str().unwrap().to_owned(),
                row["status"].as_str().unwrap().to_owned(),
                row["pass"].clone(),
            )
        })
        .collect()
}

fn listed(rows: &Value, key: &str) -> Vec<Value> {
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| row[key].clone())
        .collect()
}

#[test]
fn состояния_стенда_записаны_самим_приложением() {
    let mut programs = Vec::new();
    for (program, text) in fixtures() {
        let read = state::parse(&text).unwrap();
        assert_eq!(
            state::render(&read).unwrap(),
            text,
            "{program}: the app writes this state differently"
        );
        assert_eq!(
            read.program(),
            program,
            "the file is not named by its program"
        );
        programs.push(program);
    }
    let mut expected = vec![CHIPTUNE.to_owned(), NES_DEV.to_owned()];
    expected.sort();
    assert_eq!(programs, expected);
}

#[test]
fn стенд_несёт_каждое_состояние_этапа() {
    let shelf = stood();

    let chiptune = marks(&shelf, CHIPTUNE, "");
    let rom = marks(&shelf, NES_DEV, ROM);
    let voices = shelf
        .ask(
            "stage",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices" }),
        )
        .unwrap();

    assert_eq!(
        chiptune[0],
        ("voices".to_owned(), "opened".to_owned(), Value::Null)
    );
    assert_eq!(
        rom,
        vec![
            ("first-rom".to_owned(), "passed".to_owned(), json!("exam")),
            ("linker".to_owned(), "passed".to_owned(), json!("skip")),
        ]
    );
    let results = listed(&voices["questions"], "result");
    assert!(results.contains(&json!("miss")), "{results:?}");
    assert!(results.contains(&json!("ok")), "{results:?}");
    let blocks = listed(&voices["blocks"], "id");
    let hung = listed(&voices["clarifications"], "block");
    assert!(!hung.is_empty(), "the stand carries no clarification");
    assert!(
        hung.iter().all(|block| blocks.contains(block)),
        "a clarification hangs under no block of the stage: {hung:?}"
    );
}
