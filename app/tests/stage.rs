#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf, TOOLS, ids, uri};

#[test]
fn a_stage_carries_every_block_kind_with_pictures_as_data_uris() {
    let shelf = Shelf::new("stage");
    shelf.shelved("examples/chiptune");

    let stage = shelf
        .ask(
            "stage",
            json!({ "program": CHIPTUNE, "node": "", "stage": "voices" }),
        )
        .unwrap();

    assert_eq!(stage["id"], "voices");
    assert_eq!(stage["node"], CHIPTUNE);
    assert_eq!(stage["node_title"], "Chiptune: музыка звукового чипа NES");
    let blocks = &stage["blocks"];
    assert_eq!(
        ids(blocks, "kind"),
        [
            "heading",
            "paragraph",
            "diagram",
            "paragraph",
            "image",
            "paragraph",
            "code",
            "callout",
            "paragraph"
        ]
    );
    assert_eq!(
        ids(blocks, "id"),
        [
            "68f347b0", "874672f8", "47635068", "937ff0f4", "294ba0a9", "5a738269", "12d3e33f",
            "9bc182dd", "cd3e86d8"
        ]
    );
    assert_eq!(
        blocks[2]["src"],
        uri("image/svg+xml", "examples/chiptune/assets/voices.svg")
    );
    assert_eq!(
        blocks[4]["src"],
        uri("image/png", "examples/chiptune/assets/pulse-wave.png")
    );
    assert_eq!(blocks[4]["license"], "CC0-1.0");
    assert_eq!(blocks[4]["attribution"], "toLearn contributors");
    assert_eq!(blocks[6]["lang"], "python");
    for index in [0, 1, 3, 5, 6, 7, 8] {
        assert_eq!(blocks[index]["src"], Value::Null, "{}", blocks[index]);
    }
}

#[test]
fn practice_and_questions_are_readable_without_the_answers() {
    let shelf = Shelf::new("practice");
    shelf.shelved("examples/chiptune");

    let stage = shelf
        .ask(
            "stage",
            json!({ "program": CHIPTUNE, "node": CHIPTUNE, "stage": "voices" }),
        )
        .unwrap();

    let practice = &stage["practice"];
    assert_eq!(ids(&practice["task"], "kind"), ["paragraph", "paragraph"]);
    assert!(!practice["deliverable"].as_str().unwrap().is_empty());
    assert_eq!(ids(&practice["constraints"], "id"), ["c1", "c2"]);
    assert_eq!(ids(&practice["acceptance"], "id"), ["a1", "a2"]);
    assert!(
        !practice["acceptance"][0]["expect"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert_eq!(ids(&stage["questions"], "id"), ["q1", "q2", "q3", "q4"]);
    for question in stage["questions"].as_array().unwrap() {
        assert!(question.get("answer").is_none(), "{question}");
        assert!(!question["text"].as_str().unwrap().is_empty());
    }
}

#[test]
fn nested_stage_pictures_resolve_inside_their_subprogram() {
    let shelf = Shelf::new("nested-stage");
    shelf.shelved("fixtures/v2/valid/nes-dev");
    let home = format!("fixtures/v2/valid/nes-dev/children/{TOOLS}/children/{ROM}");

    let stage = shelf
        .ask(
            "stage",
            json!({ "program": NES_DEV, "node": ROM, "stage": "first-rom" }),
        )
        .unwrap();

    let pixel = uri("image/png", &format!("{home}/assets/pixel.png"));
    let image = stage["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["kind"] == "image")
        .unwrap();
    assert_eq!(image["src"], pixel);
    let task = stage["practice"]["task"].as_array().unwrap();
    let diagram = task
        .iter()
        .find(|block| block["kind"] == "diagram")
        .unwrap();
    assert_eq!(
        diagram["src"],
        uri("image/svg+xml", &format!("{home}/assets/pipeline.svg"))
    );
}

#[test]
fn an_ungenerated_or_foreign_stage_is_absent() {
    let shelf = Shelf::new("absent-stage");
    shelf.shelved("examples/chiptune");

    for stage in ["envelope", "../program", ""] {
        let error = shelf
            .ask(
                "stage",
                json!({ "program": CHIPTUNE, "node": "", "stage": stage }),
            )
            .unwrap_err();
        assert_eq!(error.code, "stage.absent", "{stage}");
    }
}
