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
        &json!({ "bundle": case.bundle, "query": query, "limit": 10 }),
    )
    .unwrap()
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
fn повторный_запрос_ничего_не_перечитывает() {
    let case = case("kept");
    let first = search(&case, "рантайм");

    let second = search(&case, "рантайм");

    assert_eq!(first["indexed"], 7);
    assert_eq!(second["indexed"], 0);
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

#[test]
fn индекс_с_конспектами_из_v1_не_ломает_поиск() {
    let case = case("legacy");
    search(&case, "рантайм");
    let path = case.context.search("llm-agents-base");
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replacen("kind: topic", "kind: note", 1)).unwrap();

    let out = search(&case, "локальный рантайм");

    assert_eq!(out["indexed"], 7);
    assert_eq!(first(&out)["topic"], "local-runtime");
}
