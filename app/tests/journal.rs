#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "app gate: a panic here is the report"
)]

mod support;

use serde_json::{Value, json};
use support::generating::{Case, case};
use support::speaking::{Speaking, speaking};
use tolearn_app::ipc::call;

const SAID: &str = "готов";

fn enable(case: &Case, endpoint: &str, journal: bool) {
    let http = |api| {
        json!({
            "endpoint": endpoint,
            "api": api,
            "model": "llama3:8b",
            "num_ctx": 0,
            "temperature_tenths": 7,
        })
    };
    call(
        &case.context,
        "provider",
        &json!({
            "save": {
                "enabled": true,
                "active": "local",
                "journal": journal,
                "local": http("ollama"),
                "remote": http("openai"),
                "harness": {
                    "id": "claude",
                    "command": "claude",
                    "args": ["-p"],
                    "timeout_secs": 180,
                },
            },
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": false,
        }),
    )
    .unwrap();
}

fn probe(case: &Case) -> Value {
    call(
        &case.context,
        "provider",
        &json!({
            "save": Value::Null,
            "key": Value::Null,
            "forget": false,
            "check": false,
            "probe": true,
        }),
    )
    .unwrap()
}

fn log(case: &Case, open: bool, clear: bool) -> Value {
    call(
        &case.context,
        "llm_log",
        &json!({ "open": open, "clear": clear }),
    )
    .unwrap()
}

fn heard() -> Speaking {
    speaking(|_, _| SAID.to_owned())
}

#[test]
fn выключенный_журнал_ничего_не_пишет() {
    let case = case("journal-off");
    let said = heard();
    enable(&case, &said.endpoint, false);
    probe(&case);

    assert_eq!(log(&case, false, false)["records"], json!(0));
    assert!(!case.data.join("llm-log").exists());
}

#[test]
fn включённый_журнал_пишет_запрос_и_ответ() {
    let case = case("journal-on");
    let said = heard();
    enable(&case, &said.endpoint, true);
    probe(&case);

    let out = log(&case, false, false);
    assert_eq!(out["records"], json!(1));
    assert_eq!(
        out["room"].as_str().unwrap(),
        case.data.join("llm-log").display().to_string()
    );

    let room = case.data.join("llm-log");
    let record = std::fs::read_dir(&room)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let text = std::fs::read_to_string(record).unwrap();
    assert!(text.contains("# Пробный запрос"), "{text}");
    assert!(text.contains(tolearn_provider::PROBE_PROMPT), "{text}");
    assert!(text.contains(SAID), "{text}");
}

#[test]
fn журнал_хранит_только_последние_записи() {
    let case = case("journal-rotate");
    let said = heard();
    enable(&case, &said.endpoint, true);
    for _ in 0..25 {
        probe(&case);
    }

    assert_eq!(log(&case, false, false)["records"], json!(20));
}

#[test]
fn журнал_очищается_по_команде() {
    let case = case("journal-clear");
    let said = heard();
    enable(&case, &said.endpoint, true);
    probe(&case);

    assert_eq!(log(&case, false, true)["records"], json!(0));
    assert_eq!(log(&case, false, false)["records"], json!(0));
}
