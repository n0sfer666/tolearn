#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::copied;
use tolearn_app::ipc::{Context, IpcError, call};

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-exam-{}", std::process::id())))
}

fn bundle(name: &str) -> PathBuf {
    copied(&format!("exam-{name}"))
}

fn verdict(result: &str, extra: &str) -> String {
    format!(
        "Разбор ответа.\n\n```json\n{{\n  \"topic_id\": \"local-runtime\",\n  \"verdict\": \"{result}\",\n  \"per_question\": [{{ \"id\": \"q1\", \"result\": \"ok\" }}],\n  \"gaps\": [\"квантование\"]{extra}\n}}\n```\n"
    )
}

fn parse(root: &Path, text: &str) -> Result<Value, IpcError> {
    call(
        &context(),
        "parse_verdict",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "text": text,
        }),
    )
}

fn apply(root: &Path, text: &str) -> Result<Value, IpcError> {
    call(
        &context(),
        "apply_verdict",
        &json!({
            "bundle": root.display().to_string(),
            "topic": "local-runtime",
            "text": text,
            "today": "2026-07-28",
        }),
    )
}

fn progress(root: &Path) -> String {
    std::fs::read_to_string(root.join("progress.yaml")).unwrap()
}

#[test]
fn вердикт_вставляется_целиком_а_не_выдранным_json() {
    let root = bundle("whole");

    let out = parse(&root, &verdict("pass", "")).unwrap();

    assert_eq!(out["result"], "pass");
    assert_eq!(out["gaps"][0], "квантование");
    assert_eq!(out["status"], "passed");
}

#[test]
fn разбор_ничего_не_пишет_пока_человек_не_применил() {
    let root = bundle("dry");
    let before = progress(&root);

    parse(&root, &verdict("pass", "")).unwrap();

    assert_eq!(progress(&root), before, "разбор тронул progress.yaml");
}

#[test]
fn применение_ставит_статус_по_протоколу() {
    let root = bundle("apply");

    let out = apply(&root, &verdict("pass", "")).unwrap();

    assert_eq!(out["status"], "passed");
    assert!(progress(&root).contains("passed"), "{}", progress(&root));
}

#[test]
fn неполный_вердикт_называет_пропуски_но_применяется() {
    let root = bundle("partialfields");
    let text = "```json\n{ \"topic_id\": \"local-runtime\", \"verdict\": \"partial\", \"per_question\": [{ \"id\": \"q1\", \"result\": \"partial\" }] }\n```";

    let out = parse(&root, text).unwrap();

    assert!(
        out["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "gaps"),
        "{out}"
    );
    assert_eq!(out["status"], "in_progress", "{out}");
    assert_eq!(apply(&root, text).unwrap()["status"], "in_progress");
}

#[test]
fn вердикт_о_чужой_теме_отвергается_до_записи() {
    let root = bundle("foreign");
    let before = progress(&root);
    let text = verdict("pass", "").replace("local-runtime", "cp-gateway");

    let refused = apply(&root, &text).unwrap_err();

    assert_eq!(refused.code, "verdict.wrong-topic");
    assert_eq!(progress(&root), before);
}

#[test]
fn ответ_без_json_отвергается_понятной_ошибкой() {
    let root = bundle("nojson");

    let refused = parse(&root, "Модель ответила прозой и ничего не приложила").unwrap_err();

    assert_eq!(refused.code, "verdict.no-json");
}

#[test]
fn заблокированная_тема_не_идёт_в_сделанное() {
    let root = bundle("blocked");

    apply(
        &root,
        &verdict("blocked", ", \"next_action\": \"retry_failed\""),
    )
    .unwrap();

    let tally = call(
        &context(),
        "program",
        &json!({ "bundle": root.display().to_string(), "today": "2026-07-28" }),
    )
    .unwrap();
    assert_eq!(tally["program"]["done"], 0, "{}", tally["program"]);
}

#[test]
fn три_провала_подряд_предлагают_разделить_тему() {
    let root = bundle("streak");

    let mut out = Value::Null;
    for _ in 0..3 {
        out = apply(&root, &verdict("fail", "")).unwrap();
    }

    assert_eq!(out["split_suggested"], true);
}
