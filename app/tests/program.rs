#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, call};

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-program-{}", std::process::id())))
}

fn program() -> Value {
    let root = copied("program-screen");
    call(
        &context(),
        "program",
        &json!({ "bundle": root, "today": "2026-07-27" }),
    )
    .unwrap()
}

fn topic<'a>(out: &'a Value, id: &str) -> &'a Value {
    out["topics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|topic| topic["id"] == json!(id))
        .unwrap_or_else(|| panic!("темы {id} нет в ответе: {out:#}"))
}

#[test]
fn у_темы_есть_название_и_этап_а_не_один_идентификатор() {
    let out = program();

    let first = topic(&out, "local-runtime");
    assert!(
        first["title"].as_str().unwrap().contains("рантайм"),
        "{first:#}"
    );
    assert_eq!(first["stage"], json!(1));
}

#[test]
fn чекпойнт_помечен_и_этап_называет_свой() {
    let out = program();

    assert_eq!(topic(&out, "cp-gateway")["checkpoint"], json!(true));
    assert_eq!(topic(&out, "local-runtime")["checkpoint"], json!(false));

    let stage = &out["stages"].as_array().unwrap()[0];
    assert_eq!(stage["checkpoint"], json!("cp-gateway"));
}

#[test]
fn заблокированная_тема_называет_чем_разблокируется() {
    let out = program();

    let gate = topic(&out, "cp-gateway");
    assert_eq!(gate["status"], json!("blocked"), "{gate:#}");

    let blocking: Vec<&str> = gate["blocked_by"]
        .as_array()
        .unwrap()
        .iter()
        .map(|link| link["id"].as_str().unwrap())
        .collect();
    assert!(blocking.contains(&"local-runtime"), "{gate:#}");
    assert!(
        gate["blocked_by"][0]["title"]
            .as_str()
            .unwrap()
            .chars()
            .count()
            > 3,
        "у зависимости нет названия: {gate:#}"
    );
}

#[test]
fn пройденная_зависимость_из_списка_блокировки_уходит() {
    let root = copied("program-passed");
    let progress = root.join("progress.yaml");
    let text = std::fs::read_to_string(&progress).unwrap();
    std::fs::write(
        &progress,
        text.replace(
            "  local-runtime:\n    status: todo",
            "  local-runtime:\n    status: passed",
        ),
    )
    .unwrap();

    let out = call(
        &context(),
        "program",
        &json!({ "bundle": root, "today": "2026-07-27" }),
    )
    .unwrap();

    let gate = topic(&out, "cp-gateway");
    let blocking: Vec<&str> = gate["blocked_by"]
        .as_array()
        .unwrap()
        .iter()
        .map(|link| link["id"].as_str().unwrap())
        .collect();
    assert!(
        !blocking.contains(&"local-runtime"),
        "пройденная зависимость всё ещё блокирует: {gate:#}"
    );
    assert!(!blocking.is_empty(), "{gate:#}");
}

#[test]
fn незаблокированная_тема_никого_не_ждёт() {
    let out = program();

    assert_eq!(topic(&out, "local-runtime")["blocked_by"], json!([]));
}

#[test]
fn темы_идут_в_порядке_роадмапа() {
    let out = program();

    let ids: Vec<&str> = out["topics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|topic| topic["id"].as_str().unwrap())
        .collect();
    let stages: Vec<u64> = out["topics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|topic| topic["stage"].as_u64().unwrap())
        .collect();

    assert_eq!(ids.first(), Some(&"local-runtime"), "{ids:?}");
    assert!(stages.windows(2).all(|pair| pair[0] <= pair[1]), "{ids:?}");
}
