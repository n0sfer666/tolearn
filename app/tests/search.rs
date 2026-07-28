#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};

struct Case {
    context: Context,
    bundle: String,
}

fn case(name: &str) -> Case {
    let data =
        std::env::temp_dir().join(format!("tolearn-search-ipc-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    Case {
        context: Context::new(&data),
        bundle: copied(&format!("search-{name}")).display().to_string(),
    }
}

fn search(case: &Case, query: &str) -> Value {
    call(
        &case.context,
        "search",
        &json!({ "bundle": case.bundle, "query": query, "directory": null, "limit": 10 }),
    )
    .unwrap()
}

fn note(case: &Case, body: &str) {
    call(
        &case.context,
        "save_note",
        &json!({
            "bundle": case.bundle,
            "topic": "local-runtime",
            "body": body,
            "directory": null,
            "stamp": null,
        }),
    )
    .unwrap();
}

fn hits(out: &Value) -> &Vec<Value> {
    out["hits"].as_array().unwrap()
}

fn first(out: &Value) -> &Value {
    hits(out).first().expect("выдача должна быть непустой")
}

#[test]
fn тема_находится_по_названию() {
    let case = case("topic");

    let out = search(&case, "локальный рантайм");

    assert_eq!(first(&out)["kind"], "topic");
    assert_eq!(first(&out)["topic"], "local-runtime");
}

#[test]
fn материал_находится_по_заголовку() {
    let case = case("material");

    let out = search(&case, "context length ollama");

    assert_eq!(first(&out)["kind"], "material");
    assert_eq!(first(&out)["topic"], "local-runtime");
}

#[test]
fn сохранённый_конспект_находится_следующим_запросом() {
    let case = case("note");
    note(&case, "Проверил на своём железе, слово абракадабра");

    let out = search(&case, "абракадабра");

    assert_eq!(first(&out)["kind"], "note");
    assert_eq!(first(&out)["topic"], "local-runtime");
}

#[test]
fn повторный_запрос_ничего_не_перечитывает() {
    let case = case("kept");
    let first = search(&case, "рантайм");

    let second = search(&case, "рантайм");

    assert_eq!(first["indexed"], 7);
    assert_eq!(second["indexed"], 0);
}

#[test]
fn правка_конспекта_видна_следующим_запросом() {
    let case = case("changed");
    note(&case, "Абракадабра первая");
    search(&case, "абракадабра");

    note(&case, "Тарабарщина вторая");
    let out = search(&case, "тарабарщина");

    assert_eq!(out["indexed"], 1);
    assert_eq!(first(&out)["kind"], "note");
    assert!(hits(&search(&case, "абракадабра")).is_empty());
}

#[test]
fn индекс_живёт_рядом_с_данными_а_не_в_бандле() {
    let case = case("outside");
    search(&case, "рантайм");

    let bundle: PathBuf = PathBuf::from(&case.bundle);
    let left: Vec<String> = std::fs::read_dir(&bundle)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();

    assert!(!left.iter().any(|name| name.contains("search")), "{left:?}");
}
