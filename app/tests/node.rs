#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::repository;
use support::shelf::{CHIPTUNE, NES_DEV, ROM, SOUND, Shelf, TOOLS, ids};
use tolearn_core::state::State;

#[test]
fn the_root_node_shows_its_map_with_generated_and_pending_stages() {
    let shelf = Shelf::new("root-node");
    shelf.shelved("examples/chiptune");

    let node = shelf
        .ask("node", json!({ "program": CHIPTUNE, "node": "" }))
        .unwrap();

    assert_eq!(node["program"], CHIPTUNE);
    assert_eq!(node["uuid"], CHIPTUNE);
    assert_eq!(node["title"], "Chiptune: музыка звукового чипа NES");
    assert!(node["level"].as_str().unwrap().starts_with("Начинающий"));
    assert_eq!(node["hours"], json!({ "min": 7, "max": 11 }));
    assert_eq!(node["trail"], json!([]));
    let ready: Vec<(String, bool)> = node["stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_str().unwrap().to_owned(),
                row["ready"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        ready,
        [
            ("voices".to_owned(), true),
            ("envelope".to_owned(), false),
            ("first-track".to_owned(), false),
        ]
    );
    assert_eq!(node["stages"][1]["hours"], json!({ "min": 2, "max": 4 }));
    assert_eq!(node["children"], json!([]));
}

#[test]
fn a_nested_node_carries_its_trail_and_its_own_stages() {
    let shelf = Shelf::new("nested-node");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    let root = shelf
        .ask("node", json!({ "program": NES_DEV, "node": NES_DEV }))
        .unwrap();
    let rom = shelf
        .ask("node", json!({ "program": NES_DEV, "node": ROM }))
        .unwrap();

    assert_eq!(ids(&root["children"], "id"), [TOOLS, SOUND]);
    assert_eq!(root["children"][0]["ready"], true);
    assert_eq!(root["children"][1]["ready"], false);
    assert_eq!(root["children"][1]["summary"], Value::Null);
    assert_eq!(rom["program"], NES_DEV);
    assert_eq!(rom["uuid"], ROM);
    assert_eq!(
        rom["trail"],
        json!([
            { "uuid": NES_DEV, "title": "Разработка игр для NES" },
            { "uuid": TOOLS, "title": "Инструменты сборки" },
        ])
    );
    let stages = ids(&rom["stages"], "id");
    assert!(stages.contains(&"first-rom".to_owned()), "{stages:?}");
    assert_eq!(rom["stages"][0]["ready"], true);
}

#[test]
fn an_ungenerated_subprogram_or_an_unknown_program_is_refused() {
    let shelf = Shelf::new("absent-node");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    let pending = shelf
        .ask("node", json!({ "program": NES_DEV, "node": SOUND }))
        .unwrap_err();
    let unknown = shelf
        .ask("node", json!({ "program": CHIPTUNE, "node": "" }))
        .unwrap_err();

    assert_eq!(pending.code, "node.absent");
    assert_eq!(unknown.code, "library.absent");
}

fn grow_the_sound_subprogram(shelf: &Shelf) {
    let fixture = repository().join(format!(
        "fixtures/v2/valid/nes-dev/children/{TOOLS}/children/{ROM}"
    ));
    let sound = shelf
        .data
        .join("programs")
        .join(NES_DEV)
        .join("children")
        .join(SOUND);
    let program = std::fs::read_to_string(fixture.join("program.yaml")).unwrap();
    std::fs::create_dir_all(sound.join("stages")).unwrap();
    std::fs::write(
        sound.join("program.yaml"),
        program
            .replacen(ROM, SOUND, 1)
            .replacen("slug: first-rom", "slug: sound", 1)
            .replacen("title: Первый ROM в cc65", "title: Звук и музыка", 1),
    )
    .unwrap();
    std::fs::copy(
        fixture.join("stages").join("linker.yaml"),
        sound.join("stages").join("linker.yaml"),
    )
    .unwrap();
}

#[test]
fn a_generated_subprogram_carries_the_summary_of_its_subtree() {
    let shelf = Shelf::new("subtree-node");
    shelf.shelved("fixtures/v2/valid/nes-dev");
    grow_the_sound_subprogram(&shelf);
    State::update(&shelf.data, NES_DEV, |state| {
        state.skip(ROM, "first-rom", "2026-09-10");
    })
    .unwrap();

    let root = shelf
        .ask("node", json!({ "program": NES_DEV, "node": NES_DEV }))
        .unwrap();
    let tools = shelf
        .ask("node", json!({ "program": NES_DEV, "node": TOOLS }))
        .unwrap();

    let skipped = json!({ "passed": 1, "total": 2, "skipped": 1 });
    assert_eq!(
        root["summary"],
        json!({ "passed": 1, "total": 3, "skipped": 1 })
    );
    assert_eq!(ids(&root["children"], "id"), [TOOLS, SOUND]);
    assert_eq!(root["children"][0]["summary"], skipped);
    assert_eq!(
        root["children"][1]["summary"],
        json!({ "passed": 0, "total": 1, "skipped": 0 })
    );
    assert_eq!(ids(&tools["children"], "id"), [ROM]);
    assert_eq!(tools["children"][0]["summary"], skipped);
}
