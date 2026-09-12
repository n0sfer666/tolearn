#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::json;
use support::shelf::{CHIPTUNE, Shelf};

fn shelf(name: &str) -> Shelf {
    let shelf = Shelf::new(name);
    shelf.shelved("examples/chiptune");
    shelf
}

#[test]
fn состояние_называет_вариант_сборки_и_язык_программы() {
    let out = shelf("speech-state")
        .ask("speech_state", json!({ "program": CHIPTUNE }))
        .unwrap();

    assert_eq!(out["available"], json!(cfg!(feature = "speech")));
    assert_eq!(out["listening"], json!(false));
    assert_eq!(out["language"], json!("ru"));
}

#[test]
fn расшифровка_без_записи_отвечает_кодом_а_не_текстом() {
    let error = shelf("speech-stop")
        .ask("speech_stop", json!({ "program": CHIPTUNE }))
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
    let error = shelf("speech-start")
        .ask("speech_start", json!({ "program": CHIPTUNE }))
        .unwrap_err();

    assert_eq!(error.code, "speech.off");
}

#[test]
fn чужая_программа_ловится_до_микрофона() {
    let error = Shelf::new("speech-absent")
        .ask("speech_start", json!({ "program": "nowhere-at-all" }))
        .unwrap_err();

    assert_eq!(error.code, "library.absent");
}
