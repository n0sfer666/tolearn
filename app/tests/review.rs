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
    Context::new(&std::env::temp_dir().join(format!("tolearn-review-{}", std::process::id())))
}

fn bundle(name: &str) -> PathBuf {
    copied(&format!("review-{name}"))
}

fn verdict(result: &str, missed: &str) -> String {
    format!(
        "```json\n{{ \"topic_id\": \"local-runtime\", \"verdict\": \"{result}\",\n  \"per_question\": [{{ \"id\": \"q1\", \"result\": \"miss\", \"missed\": [{missed}] }}],\n  \"gaps\": [\"квантование\"] }}\n```"
    )
}

fn apply(root: &Path, text: &str) {
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
    .unwrap();
}

fn review(root: &Path) -> Result<Value, IpcError> {
    call(
        &context(),
        "review",
        &json!({ "bundle": root.display().to_string(), "topic": "local-runtime" }),
    )
}

#[test]
fn пробел_показан_у_того_вопроса_который_его_упустил() {
    let root = bundle("attached");
    apply(&root, &verdict("fail", "\"квантование\""));

    let out = review(&root).unwrap();

    let first = &out["questions"][0];
    assert_eq!(first["id"], "q1");
    assert_eq!(first["outcome"], "miss");
    assert_eq!(first["missed"][0], "квантование");
    assert_eq!(out["loose"].as_array().unwrap().len(), 0, "{out}");
}

#[test]
fn ничей_пробел_не_теряется() {
    let root = bundle("loose");
    apply(&root, &verdict("partial", ""));

    let out = review(&root).unwrap();

    assert_eq!(out["loose"][0], "квантование");
}

#[test]
fn три_провала_подряд_дают_готовый_текст_запроса() {
    let root = bundle("split");
    for _ in 0..3 {
        apply(&root, &verdict("fail", "\"квантование\""));
    }

    let out = review(&root).unwrap();

    assert_eq!(out["split_suggested"], true);
    let request = out["split_request"].as_str().unwrap();
    assert!(request.contains("local-runtime"), "{request}");
    assert!(request.contains("квантование"), "{request}");
    assert!(request.contains("q1"), "{request}");
}

#[test]
fn до_трёх_провалов_текста_запроса_нет() {
    let root = bundle("nosplit");
    apply(&root, &verdict("fail", ""));
    apply(&root, &verdict("fail", ""));

    let out = review(&root).unwrap();

    assert_eq!(out["split_suggested"], false);
    assert_eq!(out["split_request"], "");
}

#[test]
fn прошлые_попытки_идут_строкой_а_последняя_отдельно() {
    let root = bundle("history");
    apply(&root, &verdict("fail", ""));
    apply(&root, &verdict("partial", ""));

    let out = review(&root).unwrap();

    assert_eq!(out["last"]["verdict"], "partial");
    assert_eq!(out["history"].as_array().unwrap().len(), 1);
    assert_eq!(out["history"][0]["verdict"], "fail");
}

#[test]
fn тема_без_попыток_отдаёт_вопросы_и_пустую_историю() {
    let root = bundle("fresh");

    let out = review(&root).unwrap();

    assert_eq!(out["last"], Value::Null);
    assert_eq!(out["history"].as_array().unwrap().len(), 0);
    assert!(!out["questions"].as_array().unwrap().is_empty(), "{out}");
    assert_eq!(out["questions"][0]["outcome"], Value::Null);
}

#[test]
fn разбор_ничего_не_пишет_в_бандл() {
    let root = bundle("dry");
    apply(&root, &verdict("fail", ""));
    let before = std::fs::read_to_string(root.join("progress.yaml")).unwrap();

    review(&root).unwrap();

    assert_eq!(
        std::fs::read_to_string(root.join("progress.yaml")).unwrap(),
        before
    );
}

#[test]
fn чужая_тема_отвергается() {
    let root = bundle("unknown");

    let refused = call(
        &context(),
        "review",
        &json!({ "bundle": root.display().to_string(), "topic": "нет-такой" }),
    )
    .unwrap_err();

    assert_eq!(refused.code, "topic.unknown");
}
