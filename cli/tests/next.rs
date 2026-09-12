#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "cli gate: a panic here is the report"
)]

mod generating;

use generating::Desk;
use generating::answers::{forking, told};
use serde_json::{Value, json};

#[test]
fn next_без_выбора_показывает_варианты_развилки() {
    let model = told(forking());
    let desk = Desk::new(&model);
    desk.begun();
    desk.log.clear();

    let shown = desk.run(&["next", desk.out()]).unwrap();

    assert!(
        shown.starts_with("развилка после этапа tracker:"),
        "{shown}"
    );
    assert!(shown.contains("\n  1. voices «"), "{shown}");
    assert!(
        shown.contains(
            "\n  2. noise «Шумовой канал» · 2–3 ч · рекомендую — Ударные без шума не собрать"
        ),
        "{shown}"
    );
    assert!(
        shown.contains(&format!(
            "выбор: tolearn next {} --choice <номер>",
            desk.out()
        )),
        "{shown}"
    );
    assert_eq!(desk.log.names(), ["развилка"]);
    assert!(desk.log.line("развилка").ends_with(" с · 7 → 11 ток."));
    assert_eq!(model.heard().len(), 4);
}

#[test]
fn повторный_next_берёт_открытую_развилку_без_сети_и_модели() {
    let model = told(forking());
    let mut desk = Desk::new(&model);
    desk.begun();
    let first = desk.run(&["next", desk.out()]).unwrap();
    desk.log.clear();
    desk.up = false;

    let again = desk.run(&["next", desk.out()]).unwrap();

    assert_eq!(again, first);
    assert_eq!(desk.log.text(), "");
    assert_eq!(model.heard().len(), 4);
}

#[test]
fn next_json_отдаёт_варианты_с_номерами() {
    let desk = Desk::new(&told(forking()));
    desk.begun();

    let shown = desk.run(&["next", desk.out(), "--json"]).unwrap();

    let fork: Value = serde_json::from_str(&shown).unwrap();
    let uuid = desk.program().program.uuid;
    assert_eq!(fork["program"], json!(uuid));
    assert_eq!(fork["node"], json!(uuid));
    assert_eq!(fork["after"], json!("tracker"));
    assert_eq!(fork["variants"][0]["choice"], json!(1));
    assert_eq!(fork["variants"][0]["id"], json!("voices"));
    assert_eq!(
        fork["variants"][1],
        json!({
            "choice": 2,
            "id": "noise",
            "title": "Шумовой канал",
            "hours": { "min": 2, "max": 3 },
            "why": "Ударные без шума не собрать",
            "recommended": true,
        })
    );
}

#[test]
fn next_с_выбором_строит_выбранный_этап() {
    let model = told(forking());
    let desk = Desk::new(&model);
    desk.begun();
    desk.run(&["next", desk.out()]).unwrap();
    desk.log.clear();

    let shown = desk.run(&["next", desk.out(), "--choice", "2"]).unwrap();

    let tree = desk.program();
    assert_eq!(tree.program.map.stages[1].id, "noise");
    assert_eq!(tree.stages.keys().collect::<Vec<_>>(), ["noise", "tracker"]);
    assert!(shown.contains("этап: noise · "), "{shown}");
    assert!(
        shown.contains(&format!("программа: {}", desk.path().display())),
        "{shown}"
    );
    assert_eq!(desk.log.names(), ["источники", "текст", "схемы", "запись"]);
    assert_eq!(model.heard().len(), 6);
}

#[test]
fn после_выбора_next_ведёт_от_нового_этапа() {
    let model = told(forking());
    let mut desk = Desk::new(&model);
    desk.begun();
    desk.run(&["next", desk.out()]).unwrap();
    desk.run(&["next", desk.out(), "--choice", "2"]).unwrap();
    desk.up = false;

    let error = desk
        .run(&["next", desk.out(), "--choice", "1"])
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("развилка после этапа «noise» не открыта"),
        "{error}"
    );
    assert_eq!(model.heard().len(), 6);
}
