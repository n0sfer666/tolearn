#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::json;
use support::copied;
use tolearn_app::ipc::{Context, call};

fn context() -> Context {
    Context::new(&std::env::temp_dir().join(format!("tolearn-speech-{}", std::process::id())))
}

#[test]
fn состояние_называет_вариант_сборки_и_язык_программы() {
    let bundle = copied("speech-state");
    let out = call(
        &context(),
        "speech_state",
        &json!({ "bundle": bundle.display().to_string() }),
    )
    .unwrap();

    assert_eq!(out["available"], json!(cfg!(feature = "speech")));
    assert_eq!(out["listening"], json!(false));
    assert_eq!(out["language"], json!("ru"));
}

#[test]
fn расшифровка_без_записи_отвечает_кодом_а_не_текстом() {
    let bundle = copied("speech-stop");
    let error = call(
        &context(),
        "speech_stop",
        &json!({ "bundle": bundle.display().to_string() }),
    )
    .unwrap_err();

    let expected = if cfg!(feature = "speech") {
        "speech.silent"
    } else {
        "speech.off"
    };
    assert_eq!(error.code, expected);
    assert!(!error.message.is_empty());
}

#[test]
fn выключенное_распознавание_не_открывает_микрофон() {
    if cfg!(feature = "speech") {
        return;
    }
    let bundle = copied("speech-start");
    let error = call(
        &context(),
        "speech_start",
        &json!({ "bundle": bundle.display().to_string() }),
    )
    .unwrap_err();

    assert_eq!(error.code, "speech.off");
}

#[test]
fn чужой_бандл_ловится_до_микрофона() {
    let error = call(
        &context(),
        "speech_start",
        &json!({ "bundle": "/nowhere-at-all" }),
    )
    .unwrap_err();

    assert_eq!(error.code, "scan.no-roadmap");
}
