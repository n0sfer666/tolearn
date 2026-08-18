#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use support::{copied, stub};
use tolearn_app::ipc::{Context, IpcError, call};
use tolearn_provider::{Remembered, Vault};

static CASES: AtomicUsize = AtomicUsize::new(0);

const VERDICT: &str = "{\"topic_id\": \"local-runtime\", \"verdict\": \"pass\", \"per_question\": [{\"id\": \"q1\", \"result\": \"ok\"}], \"gaps\": [\"квантование\"]}";
const SAID: &str = r#"{"message":{"content":"{\"topic_id\": \"local-runtime\", \"verdict\": \"pass\", \"per_question\": [{\"id\": \"q1\", \"result\": \"ok\"}], \"gaps\": [\"квантование\"]}"}}"#;

struct Case {
    context: Context,
    root: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!(
        "tolearn-examine-{name}-{}-{}",
        std::process::id(),
        CASES.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let vault: Arc<dyn Vault> = Arc::new(Remembered::default());
    Case {
        context: Context::with_vault(&data, vault),
        root: copied(&format!("examine-{name}")),
    }
}

fn settings(endpoint: &str, active: &str, enabled: bool) -> Value {
    let http = |api| {
        json!({
            "endpoint": endpoint,
            "api": api,
            "model": "llama3:8b",
            "num_ctx": 0,
            "temperature_tenths": 7,
        })
    };
    json!({
        "enabled": enabled,
        "active": active,
        "journal": false,
        "local": http("ollama"),
        "remote": http("openai"),
        "harness": {
            "id": "claude",
            "command": "claude",
            "args": ["-p"],
            "timeout_secs": 180,
        },
    })
}

fn enable(case: &Case, endpoint: &str, active: &str) {
    call(
        &case.context,
        "provider",
        &json!({
            "save": settings(endpoint, active, true),
            "key": if active == "remote" { json!("sk-ключ") } else { Value::Null },
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();
}

fn examine(case: &Case) -> Result<Value, IpcError> {
    call(
        &case.context,
        "examine",
        &json!({
            "bundle": case.root.display().to_string(),
            "topic": "local-runtime",
        }),
    )
}

fn parse(case: &Case, text: &str) -> Result<Value, IpcError> {
    call(
        &case.context,
        "parse_verdict",
        &json!({
            "bundle": case.root.display().to_string(),
            "topic": "local-runtime",
            "text": text,
        }),
    )
}

fn progress(root: &Path) -> String {
    std::fs::read_to_string(root.join("progress.yaml")).unwrap()
}

#[test]
fn ответ_модели_разбирается_тем_же_протоколом() {
    let case = case("same");
    let heard = stub("200 OK", SAID);
    enable(&case, &heard.endpoint, "local");

    let answer = examine(&case).unwrap();
    let out = parse(&case, answer["text"].as_str().unwrap()).unwrap();

    assert_eq!(out, parse(&case, VERDICT).unwrap());
    assert_eq!(out["result"], json!("pass"));
    assert_eq!(out["status"], json!("passed"));
}

#[test]
fn модели_уходит_тот_же_промпт() {
    let case = case("prompt");
    let heard = stub("200 OK", SAID);
    enable(&case, &heard.endpoint, "local");
    let prompt = call(
        &case.context,
        "prompt",
        &json!({ "bundle": case.root.display().to_string(), "topic": "local-runtime" }),
    )
    .unwrap();

    examine(&case).unwrap();

    let sent = heard.heard().join("\n");
    let head = prompt["text"].as_str().unwrap().lines().next().unwrap();
    assert!(sent.contains("POST /api/chat"), "{sent}");
    assert!(sent.contains(head), "{sent}");
}

#[test]
fn ответ_модели_ничего_не_пишет_в_прогресс() {
    let case = case("dry");
    let heard = stub("200 OK", SAID);
    enable(&case, &heard.endpoint, "local");
    let before = progress(&case.root);

    examine(&case).unwrap();

    assert_eq!(
        progress(&case.root),
        before,
        "экзаменатор тронул progress.yaml"
    );
}

#[test]
fn выключенный_провайдер_в_сеть_не_ходит() {
    let case = case("disabled");
    let heard = stub("200 OK", SAID);
    enable(&case, &heard.endpoint, "local");
    call(
        &case.context,
        "provider",
        &json!({
            "save": settings(&heard.endpoint, "local", false),
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();

    let failed = examine(&case).unwrap_err();

    assert_eq!(failed.code, "provider.disabled");
    assert!(heard.heard().is_empty());
}

#[test]
fn отказ_провайдера_приходит_кодом_ошибки() {
    let case = case("rejected");
    let heard = stub("401 Unauthorized", "{}");
    enable(&case, &heard.endpoint, "remote");

    let failed = examine(&case).unwrap_err();

    assert_eq!(failed.code, "provider.rejected");
}

#[test]
fn неизвестная_тема_отвергается_до_сети() {
    let case = case("topic");
    let heard = stub("200 OK", SAID);
    enable(&case, &heard.endpoint, "local");

    let failed = call(
        &case.context,
        "examine",
        &json!({
            "bundle": case.root.display().to_string(),
            "topic": "нет-такой-темы",
        }),
    )
    .unwrap_err();

    assert_eq!(failed.code, "topic.unknown");
    assert!(heard.heard().is_empty());
}
