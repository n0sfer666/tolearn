#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::shelf::{CHIPTUNE, NES_DEV, ROM, Shelf, ids};
use support::snapshot;

fn search(shelf: &Shelf, query: &str) -> Value {
    shelf
        .ask("search", json!({ "query": query, "limit": 10 }))
        .unwrap()
}

fn first(out: &Value) -> &Value {
    out["hits"]
        .as_array()
        .unwrap()
        .first()
        .expect("выдача должна быть непустой")
}

fn shown(shelf: &Shelf, hit: &Value) -> Vec<String> {
    let node = if hit["node"] == hit["program"] {
        ""
    } else {
        hit["node"].as_str().unwrap()
    };
    let stage = shelf
        .ask(
            "stage",
            json!({ "program": hit["program"], "node": node, "stage": hit["stage"] }),
        )
        .unwrap();
    let mut blocks = ids(&stage["blocks"], "id");
    blocks.extend(ids(&stage["practice"]["task"], "id"));
    blocks
}

#[test]
fn a_stage_hit_names_its_program_node_and_stage() {
    let shelf = Shelf::new("search-stage");
    shelf.shelved("examples/chiptune");

    let out = search(&shelf, "голоса чипа");

    let hit = first(&out);
    assert_eq!(hit["kind"], "stage");
    assert_eq!(hit["program"], CHIPTUNE);
    assert_eq!(hit["node"], CHIPTUNE);
    assert_eq!(hit["node_title"], "Chiptune: музыка звукового чипа NES");
    assert_eq!(hit["stage"], "voices");
    assert_eq!(hit["title"], "Голоса чипа");
    assert_eq!(hit["block"], "");
    assert_eq!(hit["snippet"], "");
}

#[test]
fn a_block_hit_points_at_a_block_the_stage_screen_shows() {
    let shelf = Shelf::new("search-block");
    shelf.shelved("examples/chiptune");

    let out = search(&shelf, "скважность периода");

    let hit = first(&out);
    assert_eq!(hit["kind"], "block");
    assert_eq!(hit["block"], "937ff0f4");
    assert!(hit["snippet"].as_str().unwrap().contains("скважность"));
    assert!(shown(&shelf, hit).contains(&"937ff0f4".to_owned()));
}

#[test]
fn a_practice_task_hit_points_at_a_task_block() {
    let shelf = Shelf::new("search-task");
    shelf.shelved("examples/chiptune");

    let out = search(&shelf, "FamiStudio");

    let hit = first(&out);
    assert_eq!(hit["block"], "15bf109d");
    assert!(shown(&shelf, hit).contains(&"15bf109d".to_owned()));
}

#[test]
fn a_child_stage_is_found_with_its_node() {
    let shelf = Shelf::new("search-child");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    let out = search(&shelf, "куда в ROM класть");

    let hit = first(&out);
    assert_eq!(hit["program"], NES_DEV);
    assert_eq!(hit["node"], ROM);
    assert_eq!(hit["node_title"], "Первый ROM в cc65");
    assert_eq!(hit["stage"], "linker");
    assert_eq!(hit["block"], "e3fd4291");
    assert!(shown(&shelf, hit).contains(&"e3fd4291".to_owned()));
}

#[test]
fn a_repeated_query_reads_nothing_again() {
    let shelf = Shelf::new("search-kept");
    shelf.shelved("examples/chiptune");
    shelf.shelved("fixtures/v2/valid/nes-dev");

    let first = search(&shelf, "канал");
    let second = search(&shelf, "канал");

    assert_eq!(first["indexed"], 2);
    assert_eq!(second["indexed"], 0);
    assert_eq!(first["hits"], second["hits"]);
}

#[test]
fn the_index_lives_beside_the_library_and_leaves_programs_alone() {
    let shelf = Shelf::new("search-outside");
    shelf.shelved("examples/chiptune");
    let programs = shelf.data.join("programs");
    let before = snapshot(&programs);

    search(&shelf, "голоса");

    assert!(shelf.context.search().is_file());
    assert_eq!(snapshot(&programs), before);
}

#[test]
fn a_v1_index_is_rebuilt() {
    let shelf = Shelf::new("search-legacy");
    shelf.shelved("examples/chiptune");
    let v1 = "schema: tolearn/search/v1\nsources:\n  - path: \"/bundle/topics/local-runtime.yaml\"\n    roadmap: \"llm-agents-base\"\n    modified: \"1\"\n    size: \"1\"\n    documents:\n      - kind: topic\n        topic: \"local-runtime\"\n        title: \"Голоса чипа\"\n        text: \"\"\n";
    std::fs::write(shelf.context.search(), v1).unwrap();

    let out = search(&shelf, "голоса чипа");

    assert_eq!(out["indexed"], 1);
    assert_eq!(first(&out)["program"], CHIPTUNE);
    let text = std::fs::read_to_string(shelf.context.search()).unwrap();
    assert!(text.starts_with("schema: tolearn/search/v2"), "{text}");
}

#[test]
fn an_empty_library_finds_nothing() {
    let shelf = Shelf::new("search-empty");

    let out = search(&shelf, "голоса");

    assert_eq!(out["hits"], json!([]));
    assert_eq!(out["indexed"], 0);
}
