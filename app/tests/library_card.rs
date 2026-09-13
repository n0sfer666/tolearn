#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf};
use tolearn_core::state::{Pass, Passed, State, key};

fn card(shelf: &Shelf, uuid: &str) -> Value {
    shelf.library()["programs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|program| program["uuid"] == uuid)
        .cloned()
        .expect("карточки программы в библиотеке нет")
}

#[test]
fn a_fresh_card_counts_generated_stages_of_the_tree_and_has_no_activity() {
    let shelf = Shelf::new("card-fresh");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    let card = card(&shelf, NES_DEV);

    assert_eq!(
        card["summary"],
        json!({ "passed": 0, "total": 2, "skipped": 0 })
    );
    assert_eq!(card["active"], Value::Null);
    assert_eq!(card["unread"], Value::Null);
    assert!(card.get("children").is_none(), "{card}");
}

#[test]
fn the_card_sums_the_whole_tree_and_dates_the_latest_activity() {
    let shelf = Shelf::new("card-tree");
    shelf.shelved("fixtures/v2/valid/nes-dev");
    State::update(&shelf.data, NES_DEV, |state| {
        let rom = state.stages.entry(key(ROM, "first-rom")).or_default();
        rom.opened = Some("2026-09-01".to_owned());
        rom.passed = Some(Passed {
            on: "2026-09-04".to_owned(),
            by: Pass::Exam,
        });
        state.stages.entry(key(ROM, "linker")).or_default().opened = Some("2026-09-02".to_owned());
    })
    .unwrap();

    let card = card(&shelf, NES_DEV);

    assert_eq!(
        card["summary"],
        json!({ "passed": 1, "total": 2, "skipped": 0 })
    );
    assert_eq!(card["active"], json!("2026-09-04"));
}

#[test]
fn a_broken_state_keeps_the_card_and_names_the_reason_instead_of_progress() {
    let shelf = Shelf::new("card-broken");
    shelf.shelved("examples/chiptune");
    let file = shelf.data.join("state").join(CHIPTUNE).join("state.yaml");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "schema: tolearn/state/9\n").unwrap();
    let refusal = shelf
        .ask("node", json!({ "program": CHIPTUNE, "node": "" }))
        .unwrap_err();

    let library = shelf.library();
    let card = card(&shelf, CHIPTUNE);

    assert_eq!(library["refused"], json!([]));
    assert_eq!(card["summary"], Value::Null);
    assert_eq!(card["active"], Value::Null);
    assert_eq!(card["unread"], json!(refusal.message));
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "schema: tolearn/state/9\n"
    );
}
