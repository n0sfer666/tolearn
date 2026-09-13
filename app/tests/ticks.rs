#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, Shelf};
use tolearn_app::ipc::{Context, call};

fn tick(shelf: &Shelf, claim: &str, on: bool) -> Value {
    shelf
        .ask(
            "tick",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices", "claim": claim, "on": on }),
        )
        .unwrap()
}

#[test]
fn галочка_пишется_в_состояние_переживает_перезапуск_и_не_трогает_статус() {
    let shelf = Shelf::new("practice-ticks");
    shelf.shelved("examples/chiptune");
    let stage = json!({ "program": CHIPTUNE, "node": "", "stage": "voices" });
    assert_eq!(
        shelf.ask("stage", stage.clone()).unwrap()["ticks"],
        json!([])
    );

    assert_eq!(tick(&shelf, "c1", true)["ticks"], json!(["c1"]));
    assert_eq!(tick(&shelf, "a1", true)["ticks"], json!(["c1", "a1"]));
    assert_eq!(tick(&shelf, "c1", false)["ticks"], json!(["a1"]));

    let restarted = Context::new(&shelf.data);
    let again = call(&restarted, "stage", &stage).unwrap();
    assert_eq!(again["ticks"], json!(["a1"]));
    let node = call(
        &restarted,
        "node",
        &json!({ "program": CHIPTUNE, "node": "" }),
    )
    .unwrap();
    assert_eq!(node["stages"][0]["status"], "opened");
    assert_eq!(node["summary"]["passed"], 0);
}

#[test]
fn галочка_чужого_пункта_отказывает_и_ничего_не_пишет() {
    let shelf = Shelf::new("practice-stray");
    shelf.shelved("examples/chiptune");

    let refused = shelf
        .ask(
            "tick",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices", "claim": "zz", "on": true }),
        )
        .unwrap_err();

    assert_eq!(refused.code, "claim.absent");
    assert!(
        !shelf.data.join("state").exists(),
        "a refused tick wrote state"
    );
}
