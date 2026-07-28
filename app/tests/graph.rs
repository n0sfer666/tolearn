#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::Path;

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};

const TODAY: &str = "2026-07-28";

fn context() -> Context {
    let directory = std::env::temp_dir().join(format!("tolearn-graph-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    Context::new(&directory)
}

fn drawn(bundle: &Path, today: &str) -> Value {
    call(
        &context(),
        "graph",
        &json!({ "bundle": bundle.display().to_string(), "today": today }),
    )
    .unwrap()
}

fn node<'a>(drawn: &'a Value, id: &str) -> &'a Value {
    drawn["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == id)
        .unwrap_or_else(|| panic!("`{id}` нет в графе: {drawn}"))
}

fn names(node: &Value, field: &str) -> Vec<String> {
    node[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn граф_отдаёт_узел_на_каждую_тему_роадмапа() {
    let bundle = copied("graph-all");

    let taken = drawn(&bundle, TODAY);

    let nodes = taken["nodes"].as_array().unwrap();
    assert!(nodes.len() >= 7, "узлов меньше, чем тем: {}", nodes.len());
    assert_eq!(node(&taken, "local-runtime")["layer"], 0);
    assert_eq!(node(&taken, "openai-compatible-api")["layer"], 1);
}

#[test]
fn узел_несёт_название_статус_и_обе_стороны_связи() {
    let bundle = copied("graph-node");

    let taken = drawn(&bundle, TODAY);

    let waiting = node(&taken, "tokens-context-cost");
    assert!(!waiting["title"].as_str().unwrap().is_empty());
    assert_eq!(waiting["status"], "blocked");
    assert_eq!(
        names(waiting, "depends_on"),
        ["local-runtime", "openai-compatible-api"]
    );
    assert_eq!(
        names(waiting, "blocked_by"),
        ["local-runtime", "openai-compatible-api"]
    );
    assert_eq!(
        names(node(&taken, "local-runtime"), "unlocks"),
        ["openai-compatible-api", "tokens-context-cost", "cp-gateway"]
    );
}

#[test]
fn зачтённая_зависимость_уходит_из_ожидания() {
    let bundle = copied("graph-passed");
    call(
        &context(),
        "set_status",
        &json!({
            "bundle": bundle.display().to_string(),
            "topic": "local-runtime",
            "status": "passed",
            "today": TODAY,
        }),
    )
    .unwrap();

    let taken = drawn(&bundle, TODAY);

    let waiting = node(&taken, "tokens-context-cost");
    assert_eq!(names(waiting, "blocked_by"), ["openai-compatible-api"]);
    assert_eq!(
        names(waiting, "depends_on"),
        ["local-runtime", "openai-compatible-api"]
    );
}

#[test]
fn узлы_идут_слоями() {
    let bundle = copied("graph-order");

    let taken = drawn(&bundle, TODAY);

    let layers: Vec<u64> = taken["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|node| node["layer"].as_u64().unwrap())
        .collect();

    assert!(
        layers.windows(2).all(|pair| pair[0] <= pair[1]),
        "{layers:?}"
    );
}

#[test]
fn граф_в_файл_не_пишет() {
    let bundle = copied("graph-quiet");
    let before = std::fs::read_to_string(bundle.join("progress.yaml")).unwrap();

    drawn(&bundle, TODAY);

    assert_eq!(
        std::fs::read_to_string(bundle.join("progress.yaml")).unwrap(),
        before
    );
}

#[test]
fn кривая_дата_отвергается_до_счёта() {
    let bundle = copied("graph-date");

    let refused = call(
        &context(),
        "graph",
        &json!({ "bundle": bundle.display().to_string(), "today": "вчера" }),
    );

    assert!(refused.is_err(), "граф посчитан по кривой дате");
}
