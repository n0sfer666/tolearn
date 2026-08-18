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
use tolearn_app::ipc::{Context, IpcError, call};

const TODAY: &str = "2026-07-29";

struct Case {
    context: Context,
    data: PathBuf,
    bundle: PathBuf,
}

fn case(name: &str) -> Case {
    let data = std::env::temp_dir().join(format!("tolearn-follow-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let bundle = copied(&format!("follow-{name}"));
    let context = Context::new(&data);
    call(
        &context,
        "import",
        &json!({ "path": bundle.display().to_string(), "today": TODAY }),
    )
    .unwrap();
    Case {
        context,
        data,
        bundle,
    }
}

impl Case {
    fn follow(&self, url: &str) -> Result<Value, IpcError> {
        call(&self.context, "follow", &json!({ "url": url }))
    }
}

#[test]
fn ссылка_приводит_к_теме_известной_программы() {
    let case = case("known");

    let out = case
        .follow("tolearn://topic?roadmap=llm-agents-base&topic=local-runtime")
        .unwrap();

    assert_eq!(
        out["program"].as_str().unwrap(),
        case.bundle.display().to_string()
    );
    assert_eq!(out["topic"], json!("local-runtime"));
}

#[test]
fn неизвестная_программа_отклоняется() {
    let case = case("no-program");

    let refused = case
        .follow("tolearn://topic?roadmap=other-program&topic=local-runtime")
        .unwrap_err();

    assert_eq!(refused.code, "link.unknown-program");
}

#[test]
fn неизвестная_тема_отклоняется() {
    let case = case("no-topic");

    let refused = case
        .follow("tolearn://topic?roadmap=llm-agents-base&topic=нет-такой")
        .unwrap_err();

    assert_eq!(refused.code, "link.value");

    let missing = case
        .follow("tolearn://topic?roadmap=llm-agents-base&topic=missing-topic")
        .unwrap_err();

    assert_eq!(missing.code, "link.unknown-topic");
}

#[test]
fn чужая_схема_до_реестра_не_доходит() {
    let case = case("scheme");

    let refused = case
        .follow("file:///etc/passwd?roadmap=llm-agents-base&topic=local-runtime")
        .unwrap_err();

    assert_eq!(refused.code, "link.scheme");
}

#[test]
fn недоступная_программа_отклоняется() {
    let case = case("unreachable");
    std::fs::remove_dir_all(&case.bundle).unwrap();

    let refused = case
        .follow("tolearn://topic?roadmap=llm-agents-base&topic=local-runtime")
        .unwrap_err();

    assert_eq!(refused.code, "link.unreachable");
}

#[test]
fn переход_ничего_не_меняет_ни_в_бандле_ни_в_реестре() {
    let case = case("read-only");
    let progress = case.bundle.join("progress.yaml");
    let registry = case.data.join("registry.yaml");
    let before = (
        std::fs::read(&progress).unwrap(),
        std::fs::read(&registry).unwrap(),
    );

    case.follow("tolearn://topic?roadmap=llm-agents-base&topic=local-runtime")
        .unwrap();

    assert_eq!(std::fs::read(&progress).unwrap(), before.0);
    assert_eq!(std::fs::read(&registry).unwrap(), before.1);
}
