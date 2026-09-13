#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf};
use tolearn_core::state::{Pass, Passed, State, key};

fn today() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    tolearn_generate::start::day(i64::try_from(now.as_secs()).unwrap())
}

fn marks(node: &Value) -> Vec<(String, String, Value)> {
    node["stages"]
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

fn pass(shelf: &Shelf, program: &str, node: &str, stage: &str, by: Pass) {
    State::update(&shelf.data, program, |state| {
        state.stages.entry(key(node, stage)).or_default().passed = Some(Passed {
            on: "2026-09-02".to_owned(),
            by,
        });
    })
    .unwrap();
}

fn opened(shelf: &Shelf, stage: &str) -> Option<String> {
    let state = State::read(&shelf.data, CHIPTUNE).unwrap();
    state
        .stages
        .get(&key(CHIPTUNE, stage))
        .and_then(|entry| entry.opened.clone())
}

#[test]
fn the_first_opening_writes_todays_date_and_a_later_one_keeps_it() {
    let shelf = Shelf::new("progress-open");
    shelf.shelved("examples/chiptune");
    let open = || {
        shelf
            .ask(
                "stage",
                json!({ "program": CHIPTUNE, "node": "", "stage": "voices" }),
            )
            .unwrap()
    };

    open();
    assert_eq!(opened(&shelf, "voices"), Some(today()));

    State::update(&shelf.data, CHIPTUNE, |state| {
        state
            .stages
            .get_mut(&key(CHIPTUNE, "voices"))
            .unwrap()
            .opened = Some("2026-01-02".to_owned());
    })
    .unwrap();
    open();
    assert_eq!(opened(&shelf, "voices"), Some("2026-01-02".to_owned()));
    assert_eq!(opened(&shelf, "envelope"), None);
}

#[test]
fn the_map_shows_every_stage_status_and_how_it_was_passed() {
    let shelf = Shelf::new("progress-map");
    shelf.shelved("examples/chiptune");
    let node = || {
        shelf
            .ask("node", json!({ "program": CHIPTUNE, "node": "" }))
            .unwrap()
    };

    let fresh = node();
    assert_eq!(
        fresh["summary"],
        json!({ "passed": 0, "total": 3, "skipped": 0 })
    );
    assert!(
        marks(&fresh)
            .iter()
            .all(|(_, status, pass)| status == "fresh" && pass.is_null())
    );

    shelf
        .ask(
            "stage",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices" }),
        )
        .unwrap();
    assert_eq!(marks(&node())[0].1, "opened");

    pass(&shelf, CHIPTUNE, CHIPTUNE, "voices", Pass::Exam);
    pass(&shelf, CHIPTUNE, CHIPTUNE, "envelope", Pass::Skip);
    let done = node();
    assert_eq!(
        marks(&done),
        [
            ("voices".to_owned(), "passed".to_owned(), json!("exam")),
            ("envelope".to_owned(), "passed".to_owned(), json!("skip")),
            ("first-track".to_owned(), "fresh".to_owned(), Value::Null),
        ]
    );
    assert_eq!(
        done["summary"],
        json!({ "passed": 2, "total": 3, "skipped": 1 })
    );
}

#[test]
fn a_container_sums_its_subtree_and_keeps_same_ids_of_other_nodes_apart() {
    let shelf = Shelf::new("progress-tree");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    pass(&shelf, NES_DEV, ROM, "first-rom", Pass::Exam);
    pass(&shelf, NES_DEV, NES_DEV, "linker", Pass::Skip);

    let root = shelf
        .ask("node", json!({ "program": NES_DEV, "node": "" }))
        .unwrap();
    let rom = shelf
        .ask("node", json!({ "program": NES_DEV, "node": ROM }))
        .unwrap();
    assert_eq!(
        root["summary"],
        json!({ "passed": 1, "total": 2, "skipped": 0 })
    );
    assert_eq!(rom["summary"], root["summary"]);
    assert_eq!(marks(&rom)[0].1, "passed");
    assert_eq!(marks(&rom)[1].1, "fresh");
}

#[test]
fn a_broken_state_refuses_with_its_code_and_stays_as_it_was() {
    let shelf = Shelf::new("progress-broken");
    shelf.shelved("examples/chiptune");
    let file = shelf.data.join("state").join(CHIPTUNE).join("state.yaml");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "schema: tolearn/state/9\n").unwrap();

    let node = shelf
        .ask("node", json!({ "program": CHIPTUNE, "node": "" }))
        .unwrap_err();
    let stage = shelf
        .ask(
            "stage",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices" }),
        )
        .unwrap_err();

    assert_eq!(node.code, "state.malformed");
    assert_eq!(stage.code, "state.malformed");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "schema: tolearn/state/9\n"
    );
}
